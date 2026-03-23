use crate::command::{
    Action, InternalAction, OnFailure, StepOutput, UserAction, WindowAction, substitute,
};
use std::os::windows::process::CommandExt;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_APP};

const WM_APP_SEQUENCE_DONE: u32 = WM_APP + 1;
const WM_APP_SEQUENCE_FAILED: u32 = WM_APP + 2;

// CREATE_NO_WINDOW — prevents a console window flashing for spawned processes
// const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn run_builtin(action: &Action, args: &[String], offset: usize, hwnd: isize) -> WindowAction {
    match action {
        Action::Internal(InternalAction::Quit) => WindowAction::Quit,
        Action::Internal(InternalAction::Hide) => WindowAction::Hide,
        Action::Internal(InternalAction::ReloadConfig) => WindowAction::Hide,
        Action::LaunchProcess {
            program,
            args: templates,
            prompts,
        } => {
            let final_args: Vec<String> = templates
                .iter()
                .map(|t| substitute(t, &args[offset..offset + prompts.len()], None))
                .collect();
            let _ = std::process::Command::new(program)
                .args(&final_args)
                // .creation_flags(CREATE_NO_WINDOW)
                .spawn();
            WindowAction::Hide
        }
        Action::OpenUrl { url, prompts } => {
            let final_url = substitute(url, &args[offset..offset + prompts.len()], None);
            open_url(&final_url);
            WindowAction::Hide
        }
        Action::Compound {
            steps,
            sequential,
            on_failure,
        } => {
            if *sequential {
                run_sequential_builtin(steps.clone(), args.to_vec(), offset, *on_failure, hwnd);
            } else {
                let mut off = offset;
                for step in steps {
                    run_builtin(step, args, off, hwnd);
                    off += step.all_prompts().len();
                }
            }
            WindowAction::Hide
        }
    }
}

pub fn run_user(action: &UserAction, args: &[String], offset: usize, hwnd: isize) -> WindowAction {
    match action {
        UserAction::LaunchProcess {
            program,
            args: templates,
            prompts,
        } => {
            let final_args: Vec<String> = templates
                .iter()
                .map(|t| substitute(t, &args[offset..offset + prompts.len()], None))
                .collect();
            let _ = std::process::Command::new(program)
                .args(&final_args)
                // .creation_flags(CREATE_NO_WINDOW)
                .spawn();
            WindowAction::Hide
        }
        UserAction::OpenUrl { url, prompts } => {
            let final_url = substitute(url, &args[offset..offset + prompts.len()], None);
            open_url(&final_url);
            WindowAction::Hide
        }
        UserAction::Compound {
            steps,
            sequential,
            on_failure,
        } => {
            eprintln!("compound: sequential={}", sequential);
            if *sequential {
                run_sequential_user(steps.clone(), args.to_vec(), offset, *on_failure, hwnd);
            } else {
                let mut off = offset;
                for step in steps {
                    run_user(step, args, off, hwnd);
                    off += step.all_prompts().len();
                }
            }
            WindowAction::Hide
        }
    }
}

// ── sequential workers ────────────────────────────────────────────────────────

fn run_sequential_builtin(
    steps: Vec<Action>,
    args: Vec<String>,
    offset: usize,
    on_failure: OnFailure,
    hwnd: isize,
) {
    std::thread::spawn(move || {
        eprintln!("worker thread started, {} steps", steps.len());
        for (i, step) in steps.iter().enumerate() {
            eprintln!("step {}: prompt_count={}", i, step.all_prompts().len());
        }
        let mut off = offset;
        let mut prev: Option<StepOutput> = None;

        for step in &steps {
            let prompt_count = step.all_prompts().len();
            let step_args = &args[off..off + prompt_count];

            let result = exec_step_builtin(step, step_args, prev.as_ref());

            match result {
                Ok(output) => {
                    prev = Some(output);
                    off += prompt_count;
                }
                Err(_) => {
                    if on_failure == OnFailure::Stop {
                        post_app_message(hwnd, WM_APP_SEQUENCE_FAILED);
                        return;
                    }
                    // continue — prev stays as last successful output
                    off += prompt_count;
                }
            }
        }

        post_app_message(hwnd, WM_APP_SEQUENCE_DONE);
    });
}

fn run_sequential_user(
    steps: Vec<UserAction>,
    args: Vec<String>,
    offset: usize,
    on_failure: OnFailure,
    hwnd: isize,
) {
    std::thread::spawn(move || {
        eprintln!("worker thread started, {} steps", steps.len());
        let mut off = offset;
        let mut prev: Option<StepOutput> = None;

        for (i, step) in steps.iter().enumerate() {
            let prompt_count = step.all_prompts().len();
            eprintln!(
                "step {}: prompt_count={}, off={}, args_len={}",
                i,
                prompt_count,
                off,
                args.len()
            );
            let step_args = &args[off..off + prompt_count];

            let result = exec_step_user(step, step_args, prev.as_ref());

            match result {
                Ok(output) => {
                    eprintln!(
                        "step {} ok, exit={}, stdout={:?}",
                        i, output.exit_code, output.stdout
                    );
                    prev = Some(output);
                    off += prompt_count;
                }
                Err(_) => {
                    eprintln!("step {} failed", i);
                    if on_failure == OnFailure::Stop {
                        post_app_message(hwnd, WM_APP_SEQUENCE_FAILED);
                        return;
                    }
                    off += prompt_count;
                }
            }
        }

        post_app_message(hwnd, WM_APP_SEQUENCE_DONE);
    });
}

// ── step executors — wait for exit, capture output ───────────────────────────

fn exec_step_builtin(
    action: &Action,
    args: &[String],
    prev: Option<&StepOutput>,
) -> Result<StepOutput, ()> {
    match action {
        Action::LaunchProcess {
            program,
            args: templates,
            prompts,
        } => {
            let final_args: Vec<String> = templates
                .iter()
                .map(|t| substitute(t, &args[..prompts.len()], prev))
                .collect();
            let output = std::process::Command::new(program)
                .args(&final_args)
                // .creation_flags(CREATE_NO_WINDOW)
                .output()
                .map_err(|_| ())?;
            let exit_code = output.status.code().unwrap_or(-1);
            if exit_code != 0 {
                return Err(());
            }
            Ok(StepOutput {
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                exit_code,
            })
        }
        Action::OpenUrl { url, prompts } => {
            // OpenUrl in sequential context — fire and treat as success
            let final_url = substitute(url, &args[..prompts.len()], prev);
            open_url(&final_url);
            Ok(StepOutput {
                stdout: String::new(),
                exit_code: 0,
            })
        }
        // Internal and nested Compound in sequential context — skip
        _ => Ok(StepOutput {
            stdout: String::new(),
            exit_code: 0,
        }),
    }
}

fn exec_step_user(
    action: &UserAction,
    args: &[String],
    prev: Option<&StepOutput>,
) -> Result<StepOutput, ()> {
    match action {
        UserAction::LaunchProcess {
            program,
            args: templates,
            prompts,
        } => {
            let final_args: Vec<String> = templates
                .iter()
                .map(|t| substitute(t, &args[..prompts.len()], prev))
                .collect();
            let output = std::process::Command::new(program)
                .args(&final_args)
                // .creation_flags(CREATE_NO_WINDOW)
                .output()
                .map_err(|_| ())?;
            let exit_code = output.status.code().unwrap_or(-1);
            if exit_code != 0 {
                return Err(());
            }
            Ok(StepOutput {
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                exit_code,
            })
        }
        UserAction::OpenUrl { url, prompts } => {
            let final_url = substitute(url, &args[..prompts.len()], prev);
            open_url(&final_url);
            Ok(StepOutput {
                stdout: String::new(),
                exit_code: 0,
            })
        }
        _ => Ok(StepOutput {
            stdout: String::new(),
            exit_code: 0,
        }),
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn open_url(url: &str) {
    let _ = std::process::Command::new("cmd")
        .args(["/c", "start", "", url])
        // .creation_flags(CREATE_NO_WINDOW)
        .spawn();
}

fn post_app_message(hwnd: isize, msg: u32) {
    unsafe {
        let _ = PostMessageW(
            Some(HWND(hwnd as *mut _)),
            msg,
            Default::default(),
            Default::default(),
        );
    }
}
