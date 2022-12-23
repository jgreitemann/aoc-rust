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
mod day12;
mod day13;
mod day14;
mod day15;
mod day16;
mod day17;
mod day18;
mod day20;
mod day21;
mod day22;
mod day23;

use aoc_companion::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    aoc_main(&[
        door!(DoorDate { day: 1, year: 2022 }, day01::Door),
        door!(DoorDate { day: 2, year: 2022 }, day02::Door),
        door!(DoorDate { day: 3, year: 2022 }, day03::Door),
        door!(DoorDate { day: 4, year: 2022 }, day04::Door),
        door!(DoorDate { day: 5, year: 2022 }, day05::Door),
        door!(DoorDate { day: 6, year: 2022 }, day06::Door),
        door!(DoorDate { day: 7, year: 2022 }, day07::Door),
        door!(DoorDate { day: 8, year: 2022 }, day08::Door),
        door!(DoorDate { day: 9, year: 2022 }, day09::Door),
        door!(DoorDate { day: 10, year: 2022 }, day10::Door),
        door!(DoorDate { day: 11, year: 2022 }, day11::Door),
        door!(DoorDate { day: 12, year: 2022 }, day12::Door),
        door!(DoorDate { day: 13, year: 2022 }, day13::Door),
        door!(DoorDate { day: 14, year: 2022 }, day14::Door),
        door!(DoorDate { day: 15, year: 2022 }, day15::Door),
        // door!(DoorDate { day: 16, year: 2022 }, day16::Door),
        door!(DoorDate { day: 17, year: 2022 }, day17::Door),
        door!(DoorDate { day: 18, year: 2022 }, day18::Door),
        door!(DoorDate { day: 20, year: 2022 }, day20::Door),
        door!(DoorDate { day: 21, year: 2022 }, day21::Door),
        door!(DoorDate { day: 22, year: 2022 }, day22::Door),
        door!(DoorDate { day: 23, year: 2022 }, day23::Door),
        // append "doors" here...
    ])
    .await
}
