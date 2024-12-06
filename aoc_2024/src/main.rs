#![allow(refining_impl_trait_internal)]

mod day01;
mod day02;
mod day03;
mod day04;
mod day05;
mod day06;

use aoc_companion::prelude::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    aoc_main(&[
        door!(DoorDate { day: 1, year: 2024 }, day01::Door),
        door!(DoorDate { day: 2, year: 2024 }, day02::Door),
        door!(DoorDate { day: 3, year: 2024 }, day03::Door),
        door!(DoorDate { day: 4, year: 2024 }, day04::Door),
        door!(DoorDate { day: 5, year: 2024 }, day05::Door),
        door!(DoorDate { day: 6, year: 2024 }, day06::Door),
        // append "doors" here...
    ])
    .await
}
