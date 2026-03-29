use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use super::Platform;
use crate::core::*;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub struct WindowsPlatform;

impl Platform for WindowsPlatform {
    fn build_command(&self, program: &str, args: &[String], ctx: &Context) -> Command {
        let mut cmd = Command::new(program);
        cmd.args(args);

        if let Some(cwd) = &ctx.cwd {
            cmd.current_dir(cwd);
        }

        #[cfg(windows)]
        {
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        cmd
    }

    fn build_shell_command(&self, command: &str, ctx: &Context) -> Command {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", command]);

        if let Some(cwd) = &ctx.cwd {
            cmd.current_dir(cwd);
        }

        #[cfg(windows)]
        {
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        cmd
    }

    fn build_open_url(&self, url: &str, ctx: &Context) -> Command {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "start", "", url]);

        if let Some(cwd) = &ctx.cwd {
            cmd.current_dir(cwd);
        }

        #[cfg(windows)]
        {
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        cmd
    }
}
