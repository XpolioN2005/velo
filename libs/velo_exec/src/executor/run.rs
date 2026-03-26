use super::{Executor, process, transform};
use crate::{core::*, executor::resolve};

impl<H: SystemHandler> Executor<H> {
    pub fn run(&self, steps: &[Step], ctx: &mut Context) -> StepResult {
        for step in steps {
            let result = match step {
                Step::Action { action, assign_to } => {
                    let result = self.run_action(action, ctx);

                    if let Some(var) = assign_to {
                        ctx.vars.insert(var.clone(), result.value.clone());
                    }

                    ctx.last = result.value.clone();
                    result
                }
            };

            if !result.success {
                return result;
            }
        }

        StepResult {
            success: true,
            value: ctx.last.clone(),
            error: None,
        }
    }

    fn run_action(&self, action: &Action, ctx: &mut Context) -> StepResult {
        match action {
            Action::LaunchProcess {
                program,
                args,
                mode,
                shell,
            } => process::run_process(program, args, mode, *shell, ctx),

            Action::OpenUrl { url } => {
                let url = resolve::resolve_string(url, ctx);

                process::run_process(
                    "start",
                    &vec!["".into(), url],
                    &ExecMode::FireForget,
                    true,
                    ctx,
                )
            }
            Action::System(id) => self.system.run(*id, ctx),
            Action::Transform(t) => transform::run_transform(t, ctx),
        }
    }
}
