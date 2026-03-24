use std::process::Command;
use std::thread;

use crate::action::{Action, Step};

pub enum ExecEvent {
    Started,
    Done,
    Failed(String),
}

pub fn run(action: &Action) -> ExecEvent {
    match action {
        Action::Internal(_) => ExecEvent::Done,

        Action::Launch { program, args } => run_launch(program, args),

        Action::OpenUrl { url } => run_url(url),

        Action::Sequence(steps) => run_sequence(steps),

        Action::Parallel(actions) => {
            run_parallel(actions);
            ExecEvent::Done
        }
    }
}

fn run_launch(program: &str, args: &[String]) -> ExecEvent {
    match Command::new(program).args(args).spawn() {
        Ok(_) => ExecEvent::Done,
        Err(e) => ExecEvent::Failed(e.to_string()),
    }
}

fn run_url(url: &str) -> ExecEvent {
    match Command::new("cmd").args(["/C", "start", "", url]).spawn() {
        Ok(_) => ExecEvent::Done,
        Err(e) => ExecEvent::Failed(e.to_string()),
    }
}

fn run_sequence(steps: &[Step]) -> ExecEvent {
    for step in steps {
        let result = if step.wait {
            run_blocking(&step.action)
        } else {
            run(&step.action)
        };

        match result {
            ExecEvent::Done => {}

            ExecEvent::Failed(e) => {
                if step.stop_on_fail {
                    return ExecEvent::Failed(e);
                }
            }

            _ => {}
        }
    }

    ExecEvent::Done
}

fn run_blocking(action: &Action) -> ExecEvent {
    match action {
        Action::Launch { program, args } => match Command::new(program).args(args).spawn() {
            Ok(mut child) => match child.wait() {
                Ok(_) => ExecEvent::Done,
                Err(e) => ExecEvent::Failed(e.to_string()),
            },
            Err(e) => ExecEvent::Failed(e.to_string()),
        },

        _ => run(action),
    }
}

fn run_parallel(actions: &[Action]) {
    for action in actions {
        let a = action.clone();

        thread::spawn(move || {
            let _ = run(&a);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::*;

    #[test]
    fn test_sequence_runs() {
        let action = Action::Sequence(vec![Step {
            action: Action::Launch {
                program: "cmd".into(),
                args: vec!["/C".into(), "echo hello".into()],
            },
            wait: true,
            stop_on_fail: true,
        }]);

        let result = run(&action);

        assert!(matches!(result, ExecEvent::Done));
    }
}
