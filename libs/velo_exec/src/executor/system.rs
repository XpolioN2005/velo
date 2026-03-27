use std::env;
use std::path::PathBuf;

use crate::core::*;

pub struct DefaultSystem;

impl SystemHandler for DefaultSystem {
    fn run(&self, action: SystemActionId, ctx: &mut Context) -> StepResult {
        match action {
            SystemActionId::GetCwd => match env::current_dir() {
                Ok(path) => StepResult {
                    success: true,
                    value: Value::String(path.to_string_lossy().to_string()),
                    error: None,
                },
                Err(e) => StepResult {
                    success: false,
                    value: Value::None,
                    error: Some(e.to_string()),
                },
            },

            SystemActionId::SetCwd => match &ctx.last {
                Value::String(path) => {
                    let pb = PathBuf::from(path);
                    if pb.exists() {
                        ctx.cwd = Some(pb);
                        StepResult {
                            success: true,
                            value: Value::None,
                            error: None,
                        }
                    } else {
                        StepResult {
                            success: false,
                            value: Value::None,
                            error: Some("Path does not exist".into()),
                        }
                    }
                }
                _ => StepResult {
                    success: false,
                    value: Value::None,
                    error: Some("SetCwd expects string in ctx.last".into()),
                },
            },

            SystemActionId::JoinPath => {
                let base = match &ctx.cwd {
                    Some(p) => p.clone(),
                    None => match env::current_dir() {
                        Ok(p) => p,
                        Err(e) => {
                            return StepResult {
                                success: false,
                                value: Value::None,
                                error: Some(e.to_string()),
                            };
                        }
                    },
                };

                match &ctx.last {
                    Value::String(seg) => {
                        let joined = base.join(seg);
                        StepResult {
                            success: true,
                            value: Value::String(joined.to_string_lossy().to_string()),
                            error: None,
                        }
                    }
                    _ => StepResult {
                        success: false,
                        value: Value::None,
                        error: Some("JoinPath expects string in ctx.last".into()),
                    },
                }
            }
        }
    }
}
