use std::{
    convert::Infallible,
    fmt::{Display, Formatter},
};
use thiserror::Error;

use anyhow::{anyhow, Result};

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
    type Error: std::error::Error + Send + Sync + 'static;

    fn into_parse_result(self) -> Result<T, Self::Error>;
}

impl<T> IntoParseResult<T> for T {
    type Error = Infallible;

    fn into_parse_result(self) -> Result<T, Self::Error> {
        Ok(self)
    }
}

impl<T, E> IntoParseResult<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    type Error = E;

    fn into_parse_result(self) -> Result<T, Self::Error> {
        self
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
impl Submissible for usize {}
impl Submissible for isize {}
impl Submissible for String {}
impl Submissible for str {}
impl Submissible for Infallible {}

pub trait IntoResult {
    type Output: ToString + Submissible;
    type Error: std::error::Error + Send + Sync + 'static;

    fn into_result(self) -> Result<Self::Output, Self::Error>;
}

impl<T> IntoResult for T
where
    T: ToString + Submissible,
{
    type Output = T;
    type Error = Infallible;

    fn into_result(self) -> Result<Self::Output, Self::Error> {
        Ok(self)
    }
}

impl<T, E> IntoResult for Result<T, E>
where
    T: ToString + Submissible,
    E: std::error::Error + Send + Sync + 'static,
{
    type Output = T;
    type Error = E;

    fn into_result(self) -> Result<Self::Output, Self::Error> {
        self
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
        time: std::time::Duration,
    },
    Skipped,
}

impl DoorPartResult {
    fn timed<T, E, F>(part_fn: F) -> Result<DoorPartResult>
    where
        T: ToString,
        E: std::error::Error + Send + Sync + 'static,
        F: FnOnce() -> Result<T, E>,
    {
        let start = std::time::Instant::now();
        let answer = part_fn()?;
        let end = std::time::Instant::now();
        Ok(DoorPartResult::Computed {
            answer: answer.to_string(),
            time: end - start,
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
                    part1: Err(anyhow::Error::from(err)),
                    part2: Err(anyhow!(DoorError::DependentParseError)),
                },
            }
        }
    }
}

#[macro_export]
macro_rules! door {
    ($date:expr, $d:ty) => {
        aoc_companion::door::DoorEntry($date, |input, parts_solved| {
            aoc_companion::door::detail::solve::<$d>(input, parts_solved)
        })
    };
}
