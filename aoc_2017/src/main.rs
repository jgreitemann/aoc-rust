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
mod day12;
mod day13;
mod day14;
mod day15;
mod day16;
mod day17;
mod day18;
mod day19;
mod day20;

use aoc_companion::prelude::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    aoc_main(&[
        door!(2017-12-01 ~> day01),
        door!(2017-12-02 ~> day02),
        door!(2017-12-03 ~> day03),
        door!(2017-12-04 ~> day04),
        door!(2017-12-05 ~> day05),
        door!(2017-12-06 ~> day06),
        door!(2017-12-07 ~> day07),
        door!(2017-12-08 ~> day08),
        door!(2017-12-09 ~> day09),
        door!(2017-12-10 ~> day10),
        door!(2017-12-11 ~> day11),
        door!(2017-12-12 ~> day12),
        door!(2017-12-13 ~> day13),
        door!(2017-12-14 ~> day14),
        door!(2017-12-15 ~> day15),
        door!(2017-12-16 ~> day16),
        door!(2017-12-17 ~> day17),
        door!(2017-12-18 ~> day18),
        door!(2017-12-19 ~> day19),
        door!(2017-12-20 ~> day20),
        // append "doors" here...
    ])
    .await
}
