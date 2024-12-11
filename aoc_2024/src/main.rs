#![allow(refining_impl_trait_internal)]

mod day01;
mod day02;
mod day03;
mod day04;
mod day05;
mod day06;
mod day07;
mod day08;
mod day09;
mod day10;
mod day11;

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
        door!(DoorDate { day: 7, year: 2024 }, day07::Door),
        door!(DoorDate { day: 8, year: 2024 }, day08::Door),
        door!(DoorDate { day: 9, year: 2024 }, day09::Door),
        door!(
            DoorDate {
                day: 10,
                year: 2024
            },
            day10::Door
        ),
        door!(
            DoorDate {
                day: 11,
                year: 2024
            },
            day11::Door
        ),
        // append "doors" here...
    ])
    .await
}
