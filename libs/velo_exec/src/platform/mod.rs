pub mod windows;
pub use windows::WindowsPlatform;

use std::process::Command;

use crate::core::*;

pub trait Platform {
    fn build_command(&self, program: &str, args: &[String], ctx: &Context) -> Command;
    fn build_shell_command(&self, command: &str, ctx: &Context) -> Command;
    fn build_open_url(&self, url: &str, ctx: &Context) -> Command;
}
