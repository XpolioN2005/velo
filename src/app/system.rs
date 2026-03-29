use velo_exec::core::{Context, StepResult, SystemHandler, Value};
use velo_exec::executor::system::DefaultSystem;

pub struct AppSystem {
    pub default: DefaultSystem,
}

impl SystemHandler for AppSystem {
    fn run(&self, action: &str, ctx: &mut Context) -> StepResult {
        let core_result = self.default.run(action, ctx);
        if core_result.success {
            return core_result;
        }

        match action {
            "internal.close_app" => StepResult {
                success: true,
                value: Value::String("quit".into()),
                error: None,
            },

            "internal.reload_config" => {
                // TODO: trigger reload logic
                StepResult {
                    success: true,
                    value: Value::None,
                    error: None,
                }
            }

            // 3. Unknown action
            _ => StepResult {
                success: false,
                value: Value::None,
                error: Some(format!("Unknown system action: {}", action)),
            },
        }
    }
}
