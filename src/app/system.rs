use velo_exec::core::{Context, StepResult, SystemActionId, SystemHandler, Value};
use velo_exec::executor::system::DefaultSystem;

// merged enum for internal use
#[derive(Copy, Clone, Debug)]
pub enum AppSystemId {
    Core(SystemActionId),
    CloseApp,
    ReloadConfig,
}

pub struct AppSystem {
    pub default: DefaultSystem,
    pub hwnd: windows::Win32::Foundation::HWND,
}

impl SystemHandler for AppSystem {
    fn run(&self, id: SystemActionId, ctx: &mut Context) -> StepResult {
        let app_id = AppSystemId::Core(id);
        self.run_extended(app_id, ctx)
    }
}

impl AppSystem {
    fn run_extended(&self, id: AppSystemId, ctx: &mut Context) -> StepResult {
        match id {
            AppSystemId::Core(core_id) => self.default.run(core_id, ctx),
            AppSystemId::CloseApp => {
                // todo
                StepResult {
                    success: true,
                    value: Value::None,
                    error: None,
                }
            }
            AppSystemId::ReloadConfig => {
                // todo
                StepResult {
                    success: true,
                    value: Value::None,
                    error: None,
                }
            }
        }
    }
}
