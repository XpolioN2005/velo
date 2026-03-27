use super::{Executor, process, transform};
use crate::{core::*, executor::resolve, platform::Platform};

impl<H: SystemHandler, P: Platform> Executor<H, P> {
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
            } => {
                let program = resolve::resolve_string(program, ctx);

                let args: Vec<String> = args
                    .iter()
                    .map(|a| resolve::resolve_string(a, ctx))
                    .collect();

                let cmd = self.platform.build_command(&program, &args, *shell, ctx);

                process::run_process(cmd, mode, ctx)
            }

            Action::OpenUrl { url } => {
                let url = resolve::resolve_string(url, ctx);

                let cmd = self
                    .platform
                    .build_command("start", &vec!["".into(), url], true, ctx);

                process::run_process(cmd, &ExecMode::FireForget, ctx)
            }

            Action::System(id) => self.system.run(*id, ctx),

            Action::Transform(t) => transform::run_transform(t, ctx),
        }
    }
}
