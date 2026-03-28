use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use regex::Regex;

use crate::core::*;

pub fn run_process(mut cmd: Command, mode: &ExecMode, ctx: &Context) -> StepResult {
    match mode {
        ExecMode::FireForget => match cmd.spawn() {
            Ok(_) => StepResult {
                success: true,
                value: ctx.last.clone(),
                error: None,
            },
            Err(e) => StepResult {
                success: false,
                value: ctx.last.clone(),
                error: Some(e.to_string()),
            },
        },

        ExecMode::Capture => match cmd.output() {
            Ok(out) => {
                let mut output = String::from_utf8_lossy(&out.stdout).to_string();

                let stderr = String::from_utf8_lossy(&out.stderr);
                if !stderr.trim().is_empty() {
                    output.push('\n');
                    output.push_str(&stderr);
                }

                StepResult {
                    success: out.status.success(),
                    value: Value::String(output),
                    error: None,
                }
            }
            Err(e) => StepResult {
                success: false,
                value: Value::None,
                error: Some(e.to_string()),
            },
        },

        ExecMode::Stream => {
            let mut child = match cmd.stdout(Stdio::piped()).spawn() {
                Ok(c) => c,
                Err(e) => {
                    return StepResult {
                        success: false,
                        value: Value::None,
                        error: Some(e.to_string()),
                    };
                }
            };

            let stdout = child.stdout.take().unwrap();
            let reader = BufReader::new(stdout);

            for line in reader.lines().flatten() {
                println!("{}", line);
            }

            let status = child.wait().ok();

            StepResult {
                success: status.map(|s| s.success()).unwrap_or(false),
                value: Value::None,
                error: None,
            }
        }

        ExecMode::StreamMatch { pattern } => {
            // compile regex at runtime
            let regex = match Regex::new(pattern) {
                Ok(r) => r,
                Err(e) => {
                    return StepResult {
                        success: false,
                        value: Value::None,
                        error: Some(format!("Invalid regex: {}", e)),
                    };
                }
            };

            let mut child = match cmd.stdout(Stdio::piped()).spawn() {
                Ok(c) => c,
                Err(e) => {
                    return StepResult {
                        success: false,
                        value: Value::None,
                        error: Some(e.to_string()),
                    };
                }
            };

            let stdout = child.stdout.take().unwrap();
            let reader = BufReader::new(stdout);

            for line in reader.lines().flatten() {
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

            StepResult {
                success: false,
                value: Value::None,
                error: Some("No match found".into()),
            }
        }
    }
}
