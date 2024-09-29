use aoc_companion::prelude::*;

use itertools::Itertools;
use thiserror::Error;

use std::str::FromStr;

pub struct Door {
    records: Vec<Record>,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("The symbol '{0}' is invalid in this context")]
    InvalidSymbol(String),
    #[error("No space on line")]
    NoSpaceOnLine,
}

impl ParseInput<'_> for Door {
    type Error = ParseError;

    fn parse(input: &str) -> Result<Self, Self::Error> {
        parse_input(input).map(|records| Self { records })
    }
}

impl Part1 for Door {
    type Output = u32;
    type Error = std::convert::Infallible;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        Ok(as_strategy(self.records.iter())
            .map(|strat| strat.score())
            .sum())
    }
}

impl Part2 for Door {
    type Output = u32;
    type Error = ParseError;

    fn part2(&self) -> Result<Self::Output, Self::Error> {
        Ok(target_strategies(self.records.iter())
            .map(|strat| strat.score())
            .sum())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hand {
    Rock,
    Paper,
    Scissors,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AmbiguousValue {
    X,
    Y,
    Z,
}

impl FromStr for AmbiguousValue {
    type Err = ParseError;

    fn from_str(c: &str) -> Result<Self, ParseError> {
        use AmbiguousValue::*;
        match c {
            "X" => Ok(X),
            "Y" => Ok(Y),
            "Z" => Ok(Z),
            _ => Err(ParseError::InvalidSymbol(c.to_owned())),
        }
    }
}

impl Hand {
    fn score(&self) -> u32 {
        match self {
            Hand::Rock => 1,
            Hand::Paper => 2,
            Hand::Scissors => 3,
        }
    }
}

impl FromStr for Hand {
    type Err = ParseError;

    fn from_str(c: &str) -> Result<Self, ParseError> {
        use Hand::*;
        match c {
            "A" => Ok(Rock),
            "B" => Ok(Paper),
            "C" => Ok(Scissors),
            _ => Err(ParseError::InvalidSymbol(c.to_owned())),
        }
    }
}

impl From<AmbiguousValue> for Hand {
    fn from(c: AmbiguousValue) -> Self {
        use AmbiguousValue::*;
        use Hand::*;
        match c {
            X => Rock,
            Y => Paper,
            Z => Scissors,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Win,
    Lose,
    Draw,
}

impl Outcome {
    fn score(&self) -> u32 {
        match self {
            Outcome::Win => 6,
            Outcome::Lose => 0,
            Outcome::Draw => 3,
        }
    }
}

impl From<AmbiguousValue> for Outcome {
    fn from(c: AmbiguousValue) -> Self {
        use AmbiguousValue::*;
        use Outcome::*;
        match c {
            X => Lose,
            Y => Draw,
            Z => Win,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Record {
    theirs: Hand,
    ambigous: AmbiguousValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Strategy {
    theirs: Hand,
    ours: Hand,
}

impl Strategy {
    fn outcome(&self) -> Outcome {
        use Hand::*;
        match self {
            Strategy {
                theirs: Rock,
                ours: Paper,
            } => Outcome::Win,
            Strategy {
                theirs: Rock,
                ours: Scissors,
            } => Outcome::Lose,
            Strategy {
                theirs: Paper,
                ours: Rock,
            } => Outcome::Lose,
            Strategy {
                theirs: Paper,
                ours: Scissors,
            } => Outcome::Win,
            Strategy {
                theirs: Scissors,
                ours: Rock,
            } => Outcome::Win,
            Strategy {
                theirs: Scissors,
                ours: Paper,
            } => Outcome::Lose,
            Strategy { theirs, ours } if ours == theirs => Outcome::Draw,
            _ => unreachable!(),
        }
    }

    fn to_achieve_outcome(theirs: Hand, desired: Outcome) -> Strategy {
        let ours = [Hand::Rock, Hand::Paper, Hand::Scissors]
            .into_iter()
            .find(|&ours| Strategy { theirs, ours }.outcome() == desired)
            .unwrap();
        Strategy { theirs, ours }
    }

    fn score(&self) -> u32 {
        self.ours.score() + self.outcome().score()
    }
}

fn parse_input(input: &str) -> Result<Vec<Record>, ParseError> {
    input
        .lines()
        .map(|line| line.split_once(' ').ok_or(ParseError::NoSpaceOnLine))
        .map_ok(|(theirs_str, ambiguous_str)| {
            Ok(Record {
                theirs: theirs_str.parse()?,
                ambigous: ambiguous_str.parse()?,
            })
        })
        .flatten()
        .collect()
}

fn as_strategy<'r>(
    records: impl Iterator<Item = &'r Record> + 'r,
) -> impl Iterator<Item = Strategy> + 'r {
    records.map(|&Record { theirs, ambigous }| Strategy {
        theirs,
        ours: ambigous.into(),
    })
}

fn target_strategies<'r>(
    records: impl Iterator<Item = &'r Record> + 'r,
) -> impl Iterator<Item = Strategy> + 'r {
    records
        .map(|&Record { theirs, ambigous }| Strategy::to_achieve_outcome(theirs, ambigous.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::assert_equal;

    const EXAMPLE_INPUT: &str = r"A Y
B X
C Z";

    const EXAMPLE_GUIDE: &[Strategy] = &[
        Strategy {
            theirs: Hand::Rock,
            ours: Hand::Paper,
        },
        Strategy {
            theirs: Hand::Paper,
            ours: Hand::Rock,
        },
        Strategy {
            theirs: Hand::Scissors,
            ours: Hand::Scissors,
        },
    ];

    const TARGET_STRATS: &[Strategy] = &[
        Strategy {
            theirs: Hand::Rock,
            ours: Hand::Rock,
        },
        Strategy {
            theirs: Hand::Paper,
            ours: Hand::Rock,
        },
        Strategy {
            theirs: Hand::Scissors,
            ours: Hand::Rock,
        },
    ];

    #[test]
    fn strategy_guide_is_parsed() {
        assert_equal(
            as_strategy(parse_input(EXAMPLE_INPUT).unwrap().iter()),
            EXAMPLE_GUIDE.iter().copied(),
        );
    }

    #[test]
    fn strategy_scores_are_calculated() {
        assert_equal(EXAMPLE_GUIDE.iter().map(Strategy::score), [8, 1, 6])
    }

    #[test]
    fn target_strategies_are_found() {
        assert_equal(
            target_strategies(parse_input(EXAMPLE_INPUT).unwrap().iter()),
            TARGET_STRATS.iter().copied(),
        );
    }
}
