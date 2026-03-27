use std::process::Command;

use super::Platform;
use crate::core::*;

pub struct WindowsPlatform;

impl Platform for WindowsPlatform {
    fn build_command(&self, program: &str, args: &[String], shell: bool, ctx: &Context) -> Command {
        let mut cmd = if shell {
            let full = format!("{} {}", program, args.join(" "));
            let mut c = Command::new("cmd");
            c.args(["/C", &full]);
            c
        } else {
            let mut c = Command::new(program);
            c.args(args);
            c
        };

        if let Some(cwd) = &ctx.cwd {
            cmd.current_dir(cwd);
        }

        cmd
    }
}
