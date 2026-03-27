use std::process::Command;

use super::Platform;
use crate::core::*;

pub struct WindowsPlatform;

impl Platform for WindowsPlatform {
    fn build_command(&self, program: &str, args: &[String], ctx: &Context) -> Command {
        let mut cmd = Command::new(program);
        cmd.args(args);

        if let Some(cwd) = &ctx.cwd {
            cmd.current_dir(cwd);
        }

        cmd
    }

    fn build_shell_command(&self, command: &str, ctx: &Context) -> Command {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", command]);

        if let Some(cwd) = &ctx.cwd {
            cmd.current_dir(cwd);
        }

        cmd
    }

    fn build_open_url(&self, url: &str, ctx: &Context) -> Command {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "start", "", url]);

        if let Some(cwd) = &ctx.cwd {
            cmd.current_dir(cwd);
        }

        cmd
    }
}
