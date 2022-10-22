mod api;
pub mod door;
mod output;
mod runtime;
mod validation;

pub mod prelude {
    pub use crate::door;
    pub use crate::door::{DoorDate, DoorEntry, ParseInput, Part1, Part2};
    pub use crate::runtime::aoc_main;
    pub use anyhow::Result;
    pub use thiserror::Error;
}
pub use prelude::*;
