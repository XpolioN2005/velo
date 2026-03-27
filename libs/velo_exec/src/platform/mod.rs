pub mod windows;
pub use windows::WindowsPlatform;

use std::process::Command;

use crate::core::*;

pub trait Platform {
    fn build_command(&self, program: &str, args: &[String], shell: bool, ctx: &Context) -> Command;
}
