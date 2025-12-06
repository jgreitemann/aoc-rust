use std::{
    convert::Infallible,
    fmt::{Display, Formatter},
};
use thiserror::Error;

use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DoorDate {
    pub day: u32,
    pub year: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    Part1,
    Part2,
}

impl Display for Part {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Part::Part1 => "1",
            Part::Part2 => "2",
        })
    }
}

#[derive(Error, Debug)]
pub enum DoorError {
    #[error("solution for this part is not yet implemented")]
    SolutionNotImplemented,
    #[error("could not parse input for this door; see prev. error")]
    DependentParseError,
}

pub trait IntoParseResult<T> {
    fn into_parse_result(self) -> anyhow::Result<T>;
}

impl<T> IntoParseResult<T> for T {
    fn into_parse_result(self) -> Result<T> {
        Ok(self)
    }
}

impl<T, E> IntoParseResult<T> for Result<T, E>
where
    anyhow::Error: From<E>,
{
    fn into_parse_result(self) -> Result<T> {
        Ok(self?)
    }
}

pub trait Submissible {}

impl Submissible for u8 {}
impl Submissible for i8 {}
impl Submissible for u16 {}
impl Submissible for i16 {}
impl Submissible for u32 {}
impl Submissible for i32 {}
impl Submissible for u64 {}
impl Submissible for i64 {}
impl Submissible for u128 {}
impl Submissible for i128 {}
impl Submissible for usize {}
impl Submissible for isize {}
impl Submissible for String {}
impl Submissible for str {}
impl Submissible for Infallible {}

pub trait IntoResult {
    type Output: ToString + Submissible;

    fn into_result(self) -> Result<Self::Output>;
}

impl<T> IntoResult for T
where
    T: ToString + Submissible,
{
    type Output = T;

    fn into_result(self) -> Result<Self::Output> {
        Ok(self)
    }
}

impl<T, E> IntoResult for Result<T, E>
where
    anyhow::Error: From<E>,
    T: ToString + Submissible,
{
    type Output = T;

    fn into_result(self) -> Result<Self::Output> {
        Ok(self?)
    }
}

pub trait Solution<'input>: Sized {
    fn parse(input: &'input str) -> impl IntoParseResult<Self>;

    fn part1(&self) -> impl IntoResult {
        Err::<Infallible, DoorError>(DoorError::SolutionNotImplemented)
    }

    fn part2(&self) -> impl IntoResult {
        Err::<Infallible, DoorError>(DoorError::SolutionNotImplemented)
    }
}

pub struct DoorEntry(pub DoorDate, pub fn(&str, usize) -> DoorResult);

#[derive(Debug, PartialEq)]
pub enum DoorPartResult {
    Computed {
        answer: String,
        time: time::Duration,
    },
    Skipped,
}

impl DoorPartResult {
    fn timed<T, F>(part_fn: F) -> Result<DoorPartResult>
    where
        T: ToString,
        F: FnOnce() -> Result<T>,
    {
        let start = std::time::Instant::now();
        let answer = part_fn()?;
        let end = std::time::Instant::now();
        Ok(DoorPartResult::Computed {
            answer: answer.to_string(),
            time: (end - start)
                .try_into()
                .expect("duration should be positive"),
        })
    }
}

#[derive(Debug)]
pub struct DoorResult {
    pub part1: Result<DoorPartResult>,
    pub part2: Result<DoorPartResult>,
}

pub mod detail {
    use super::*;

    pub fn solve<'input, D: Solution<'input>>(
        input: &'input str,
        parts_solved: usize,
    ) -> DoorResult {
        if parts_solved >= 2 {
            DoorResult {
                part1: Ok(DoorPartResult::Skipped),
                part2: Ok(DoorPartResult::Skipped),
            }
        } else {
            match D::parse(input).into_parse_result() {
                Ok(door) => {
                    if parts_solved == 0 {
                        DoorResult {
                            part1: DoorPartResult::timed(|| door.part1().into_result()),
                            part2: DoorPartResult::timed(|| door.part2().into_result()),
                        }
                    } else {
                        DoorResult {
                            part1: Ok(DoorPartResult::Skipped),
                            part2: DoorPartResult::timed(|| door.part2().into_result()),
                        }
                    }
                }
                Err(err) => DoorResult {
                    part1: Err(err),
                    part2: Err(anyhow!(DoorError::DependentParseError)),
                },
            }
        }
    }
}

#[macro_export]
macro_rules! door {
    ($year:literal-12-$day:literal ~> $mod:ident) => {
        aoc_companion::door::DoorEntry(
            aoc_companion::door::DoorDate {
                #[allow(clippy::zero_prefixed_literal)]
                day: $day,
                year: $year,
            },
            |input, parts_solved| {
                aoc_companion::door::detail::solve::<$mod::Door>(input, parts_solved)
            },
        )
    };
}
