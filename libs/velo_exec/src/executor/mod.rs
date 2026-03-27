pub mod process;
pub mod resolve;
pub mod run;
pub mod system;
pub mod transform;

use crate::core::*;
use crate::platform::Platform;

pub struct Executor<H: SystemHandler, P: Platform> {
    pub system: H,
    pub platform: P,
}

impl<H: SystemHandler, P: Platform> Executor<H, P> {
    pub fn new(system: H, platform: P) -> Self {
        Self { system, platform }
    }
}
