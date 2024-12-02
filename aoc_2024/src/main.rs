#![allow(refining_impl_trait_internal)]

mod day01;
mod day02;

use aoc_companion::prelude::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    aoc_main(&[
        door!(DoorDate { day: 1, year: 2024 }, day01::Door),
        door!(DoorDate { day: 2, year: 2024 }, day02::Door),
        // append "doors" here...
    ])
    .await
}
