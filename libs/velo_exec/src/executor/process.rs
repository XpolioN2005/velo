use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use super::resolve;
use crate::core::*;

pub fn run_process(program: &str, args: &[String], mode: &ExecMode, ctx: &Context) -> StepResult {
    let program = resolve::resolve_string(program, ctx);
    let args = resolve::resolve_args(args, ctx);

    let mut cmd = Command::new(program);
    cmd.args(args);

    if let Some(cwd) = &ctx.cwd {
        cmd.current_dir(cwd);
    }

    match mode {
        ExecMode::FireForget => match cmd.spawn() {
            Ok(_) => StepResult {
                success: true,
                value: ctx.last.clone(),
                error: None,
            },
            Err(_) => StepResult {
                success: false,
                value: Value::None,
                error: None,
            },
        },

        ExecMode::Capture => match cmd.output() {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                StepResult {
                    success: out.status.success(),
                    value: Value::String(stdout),
                    error: None,
                }
            }
            Err(_) => StepResult {
                success: false,
                value: Value::None,
                error: None,
            },
        },

        ExecMode::Stream => {
            let mut child = match cmd.stdout(Stdio::piped()).spawn() {
                Ok(c) => c,
                Err(_) => {
                    return StepResult {
                        success: false,
                        value: Value::None,
                        error: None,
                    };
                }
            };

            let stdout = child.stdout.take().unwrap();
            let reader = BufReader::new(stdout);

            for line in reader.lines() {
                if let Ok(line) = line {
                    println!("{}", line); // temporary
                }
            }

            let status = child.wait().ok();
            StepResult {
                success: status.map(|s| s.success()).unwrap_or(false),
                value: Value::None,
                error: None,
            }
        }

        ExecMode::StreamMatch(regex) => {
            let mut child = match cmd.stdout(Stdio::piped()).spawn() {
                Ok(c) => c,
                Err(_) => {
                    return StepResult {
                        success: false,
                        value: Value::None,
                        error: None,
                    };
                }
            };

            let stdout = child.stdout.take().unwrap();
            let reader = BufReader::new(stdout);

            for line in reader.lines() {
                if let Ok(line) = line {
                    if let Some(caps) = regex.captures(&line) {
                        if let Some(m) = caps.get(1) {
                            return StepResult {
                                success: true,
                                value: Value::String(m.as_str().to_string()),
                                error: None,
                            };
                        }
                    }
                }
            }

            StepResult {
                success: false,
                value: Value::None,
                error: None,
            }
        }
    }
}
