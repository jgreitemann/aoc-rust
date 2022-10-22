mod day01;
mod day02;
mod day03;
mod day04;
mod day05;

use aoc_companion::prelude::*;


#[tokio::main]
async fn main() -> Result<()> {
    aoc_main(&[
        door!(DoorDate { day: 1, year: 2017 }, day01::Door),
        door!(DoorDate { day: 2, year: 2017 }, day02::Door),
        door!(DoorDate { day: 3, year: 2017 }, day03::Door),
        door!(DoorDate { day: 4, year: 2017 }, day04::Door),
        door!(DoorDate { day: 5, year: 2017 }, day05::Door),
        // append "doors" here...
        ]).await
}