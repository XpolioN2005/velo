pub mod process;
pub mod resolve;
pub mod run;
pub mod system;
pub mod transform;

use crate::core::*;

pub struct Executor<H: SystemHandler> {
    pub system: H,
}

impl<H: SystemHandler> Executor<H> {
    pub fn new(system: H) -> Self {
        Self { system }
    }
}
