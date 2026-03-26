use super::resolve;
use crate::{core::*, executor::resolve::value_to_string};

pub fn run_transform(t: &Transform, ctx: &Context) -> StepResult {
    match t {
        Transform::Regex {
            input,
            pattern,
            group,
        } => {
            let input = match input {
                Some(s) => resolve::resolve_string(s, ctx),
                None => value_to_string(&ctx.last),
            };

            let re = match regex::Regex::new(pattern) {
                Ok(r) => r,
                Err(e) => {
                    return StepResult {
                        success: false,
                        value: Value::None,
                        error: Some(e.to_string()),
                    };
                }
            };

            if let Some(caps) = re.captures(&input) {
                if let Some(m) = caps.get(*group) {
                    return StepResult {
                        success: true,
                        value: Value::String(m.as_str().to_string()),
                        error: None,
                    };
                }
            }

            StepResult {
                success: false,
                value: Value::None,
                error: None,
            }
        }

        // ── SPLIT ─────────────────────────
        Transform::Split { input, delimiter } => {
            let src = match input {
                Some(s) => resolve::resolve_string(s, ctx),
                None => value_to_string(&ctx.last),
            };

            let parts = src
                .split(delimiter)
                .map(|s| Value::String(s.to_string()))
                .collect::<Vec<_>>();

            StepResult {
                success: true,
                value: Value::List(parts),
                error: None,
            }
        }

        // ── FIRST ─────────────────────────
        Transform::First { .. } => match &ctx.last {
            Value::List(list) => {
                if let Some(first) = list.first() {
                    StepResult {
                        success: true,
                        value: first.clone(),
                        error: None,
                    }
                } else {
                    StepResult {
                        success: false,
                        value: Value::None,
                        error: Some("Empty list".into()),
                    }
                }
            }
            _ => StepResult {
                success: false,
                value: Value::None,
                error: Some("First expects list".into()),
            },
        },
    }
}
