#![allow(refining_impl_trait_internal)]

mod day01;
mod day02;
mod day03;
mod day04;
mod day05;
mod day06;
mod day07;
mod day08;
// mod day09;
// mod day10;
// mod day11;
// mod day12;

use aoc_companion::prelude::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    aoc_main(&[
        door!(2025-12-01 ~> day01),
        door!(2025-12-02 ~> day02),
        door!(2025-12-03 ~> day03),
        door!(2025-12-04 ~> day04),
        door!(2025-12-05 ~> day05),
        door!(2025-12-06 ~> day06),
        door!(2025-12-07 ~> day07),
        door!(2025-12-08 ~> day08),
        // door!(2025-12-09 ~> day09),
        // door!(2025-12-10 ~> day10),
        // door!(2025-12-11 ~> day11),
        // door!(2025-12-12 ~> day12),
    ])
    .await
}
