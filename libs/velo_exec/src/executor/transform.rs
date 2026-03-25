use super::resolve;
use crate::core::*;

pub fn run_transform(t: &Transform, ctx: &Context) -> StepResult {
    match t {
        Transform::Regex {
            input,
            pattern,
            group,
        } => {
            let input = resolve::resolve_string(input, ctx);

            let re = match regex::Regex::new(pattern) {
                Ok(r) => r,
                Err(_) => {
                    return StepResult {
                        success: false,
                        value: Value::None,
                        error: None,
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
    }
}
