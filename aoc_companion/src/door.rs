use std::fmt::{Display, Formatter};
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

pub trait ParseInput<'input>: Sized {
    type Error: std::error::Error + Send + Sync + 'static;
    fn parse(input: &'input str) -> Result<Self, Self::Error>;
}

pub trait Part1 {
    type Output: ToString;
    type Error: std::error::Error + Send + Sync + 'static;
    fn part1(&self) -> Result<Self::Output, Self::Error>;
}

pub trait Part2 {
    type Output: ToString;
    type Error: std::error::Error + Send + Sync + 'static;
    fn part2(&self) -> Result<Self::Output, Self::Error>;
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

pub struct DoorResult {
    pub part1: Result<DoorPartResult>,
    pub part2: Result<DoorPartResult>,
}

pub mod detail {
    use super::*;
    pub use core::marker::PhantomData;

    // http://lukaskalbertodt.github.io/2019/12/05/generalized-autoref-based-specialization.html
    pub struct AutoRefSpecializationWrapper<T>(pub PhantomData<T>);

    pub trait DoorSolution<'input> {
        fn solve(&self, input: &'input str, parts_solved: usize) -> DoorResult;
    }

    impl<'input, D> DoorSolution<'input> for AutoRefSpecializationWrapper<D>
    where
        D: ParseInput<'input> + Part1,
    {
        fn solve(&self, input: &'input str, parts_solved: usize) -> DoorResult {
            DoorResult {
                part1: if parts_solved == 0 {
                    match D::parse(input) {
                        Ok(door) => DoorPartResult::timed(|| door.part1()),
                        Err(err) => Err(anyhow::Error::from(err)),
                    }
                } else {
                    Ok(DoorPartResult::Skipped)
                },
                part2: Err(anyhow!(DoorError::SolutionNotImplemented)),
            }
        }
    }

    impl<'input, D> DoorSolution<'input> for &AutoRefSpecializationWrapper<D>
    where
        D: ParseInput<'input> + Part1 + Part2,
    {
        fn solve(&self, input: &'input str, parts_solved: usize) -> DoorResult {
            if parts_solved >= 2 {
                DoorResult {
                    part1: Ok(DoorPartResult::Skipped),
                    part2: Ok(DoorPartResult::Skipped),
                }
            } else {
                match D::parse(input) {
                    Ok(door) => {
                        if parts_solved == 0 {
                            DoorResult {
                                part1: DoorPartResult::timed(|| door.part1()),
                                part2: DoorPartResult::timed(|| door.part2()),
                            }
                        } else {
                            DoorResult {
                                part1: Ok(DoorPartResult::Skipped),
                                part2: DoorPartResult::timed(|| door.part2()),
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
}

#[macro_export]
macro_rules! door {
    ($date:expr, $d:ty) => {
        aoc_companion::door::DoorEntry($date, |input, parts_solved| {
            use aoc_companion::door::detail::*;
            (&&AutoRefSpecializationWrapper(std::marker::PhantomData::<$d>))
                .solve(input, parts_solved)
        })
    };
}
