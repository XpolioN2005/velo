use super::{Executor, process, transform};
use crate::{
    core::*,
    executor::resolve::{self, resolve_args},
    platform::Platform,
};

impl<H: SystemHandler, P: Platform> Executor<H, P> {
    pub fn run(&self, steps: &[Step], ctx: &mut Context) -> StepResult {
        for step in steps {
            let result = self.run_action(&step.action, ctx);

            // assign only on success
            if result.success {
                if let Some(var) = &step.assign_to {
                    ctx.vars.insert(var.clone(), result.value.clone());
                }
                ctx.last = result.value.clone();
            }

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
            Action::Process {
                program,
                args,
                mode,
            } => {
                let program = resolve::resolve_string(program, ctx);
                let args: Vec<String> = resolve_args(args, ctx);

                let cmd = self.platform.build_command(&program, &args, ctx);
                process::run_process(cmd, mode, ctx)
            }

            Action::Shell { command, mode } => {
                let command = resolve::resolve_string(command, ctx);

                let cmd = self.platform.build_shell_command(&command, ctx);
                process::run_process(cmd, mode, ctx)
            }

            Action::OpenUrl { url } => {
                let url = resolve::resolve_string(url, ctx);

                let cmd = self.platform.build_open_url(&url, ctx);
                process::run_process(cmd, &ExecMode::FireForget, ctx)
            }

            Action::System { action } => self.system.run(action, ctx),

            Action::Transform(t) => transform::run_transform(t, ctx),
        }
    }
}
