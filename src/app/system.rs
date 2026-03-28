use velo_exec::core::{Context, StepResult, SystemHandler, Value};
use velo_exec::executor::system::DefaultSystem;

pub struct AppSystem {
    pub default: DefaultSystem,
    pub hwnd: windows::Win32::Foundation::HWND,
}

impl SystemHandler for AppSystem {
    fn run(&self, action: &str, ctx: &mut Context) -> StepResult {
        let core_result = self.default.run(action, ctx);
        if core_result.success {
            return core_result;
        }

        match action {
            "internal.close_app" => {
                // TODO: actually close using hwnd
                // Example (later):
                // unsafe { PostMessageW(self.hwnd, WM_CLOSE, ...); }

                StepResult {
                    success: true,
                    value: Value::None,
                    error: None,
                }
            }

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
