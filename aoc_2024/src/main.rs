#![allow(refining_impl_trait_internal)]

mod day01;

use aoc_companion::prelude::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    aoc_main(&[
        door!(DoorDate { day: 1, year: 2024 }, day01::Door),
        // append "doors" here...
    ])
    .await
}
