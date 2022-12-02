use aoc_companion::prelude::*;

use thiserror::Error;

pub struct Door {
    rules: Vec<Rule>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("The symbol '{0}' is invalid in this context")]
    InvalidSymbol(String),
    #[error("No space on line")]
    NoSpaceOnLine,
}

impl ParseInput<'_> for Door {
    type Error = Error;

    fn parse(input: &str) -> Result<Self, Self::Error> {
        parse_input(input).map(|rules| Self { rules })
    }
}

impl Part1 for Door {
    type Output = u32;
    type Error = std::convert::Infallible;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        Ok(as_guide(self.rules.iter()).map(|strat| strat.score()).sum())
    }
}

impl Part2 for Door {
    type Output = u32;
    type Error = Error;

    fn part2(&self) -> Result<Self::Output, Self::Error> {
        Ok(target_strategies(self.rules.iter())
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

impl AmbiguousValue {
    fn from_str(c: &str) -> Result<Self, Error> {
        use AmbiguousValue::*;
        match c {
            "X" => Ok(X),
            "Y" => Ok(Y),
            "Z" => Ok(Z),
            _ => Err(Error::InvalidSymbol(c.to_owned())),
        }
    }
}

impl Hand {
    fn from_str(c: &str) -> Result<Self, Error> {
        use Hand::*;
        match c {
            "A" => Ok(Rock),
            "B" => Ok(Paper),
            "C" => Ok(Scissors),
            _ => Err(Error::InvalidSymbol(c.to_owned())),
        }
    }

    fn score(&self) -> u32 {
        match self {
            Hand::Rock => 1,
            Hand::Paper => 2,
            Hand::Scissors => 3,
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
struct Rule {
    opponent_will_play: Hand,
    ambigous: AmbiguousValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Strategy {
    opponent_will_play: Hand,
    i_should_play: Hand,
}

impl Strategy {
    fn outcome(&self) -> Outcome {
        use Hand::*;
        match self {
            Strategy {
                opponent_will_play: Rock,
                i_should_play: Paper,
            } => Outcome::Win,
            Strategy {
                opponent_will_play: Rock,
                i_should_play: Scissors,
            } => Outcome::Lose,
            Strategy {
                opponent_will_play: Paper,
                i_should_play: Rock,
            } => Outcome::Lose,
            Strategy {
                opponent_will_play: Paper,
                i_should_play: Scissors,
            } => Outcome::Win,
            Strategy {
                opponent_will_play: Scissors,
                i_should_play: Rock,
            } => Outcome::Win,
            Strategy {
                opponent_will_play: Scissors,
                i_should_play: Paper,
            } => Outcome::Lose,
            Strategy {
                opponent_will_play,
                i_should_play,
            } if opponent_will_play == i_should_play => Outcome::Draw,
            _ => unreachable!(),
        }
    }

    fn to_achieve_outcome(opponent_will_play: Hand, desired: Outcome) -> Strategy {
        let i_should_play = [Hand::Rock, Hand::Paper, Hand::Scissors]
            .into_iter()
            .find(|i_should_play| {
                Strategy {
                    opponent_will_play,
                    i_should_play: *i_should_play,
                }
                .outcome()
                    == desired
            })
            .unwrap();
        Strategy {
            opponent_will_play,
            i_should_play,
        }
    }

    fn score(&self) -> u32 {
        self.i_should_play.score() + self.outcome().score()
    }
}

fn parse_input(input: &str) -> Result<Vec<Rule>, Error> {
    input
        .lines()
        .map(|line| line.split_once(' ').ok_or(Error::NoSpaceOnLine))
        .map(|res| {
            let (op_str, other) = res?;
            Ok(Rule {
                opponent_will_play: Hand::from_str(op_str)?,
                ambigous: AmbiguousValue::from_str(other)?,
            })
        })
        .collect()
}

fn as_guide<'r>(rules: impl Iterator<Item = &'r Rule> + 'r) -> impl Iterator<Item = Strategy> + 'r {
    rules.map(
        |&Rule {
             opponent_will_play,
             ambigous,
         }| {
            Strategy {
                opponent_will_play,
                i_should_play: ambigous.into(),
            }
        },
    )
}

fn target_strategies<'r>(
    rules: impl Iterator<Item = &'r Rule> + 'r,
) -> impl Iterator<Item = Strategy> + 'r {
    rules.into_iter().map(
        |&Rule {
             opponent_will_play,
             ambigous,
         }| {
            let desired: Outcome = ambigous.into();
            Strategy::to_achieve_outcome(opponent_will_play, desired)
        },
    )
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
            opponent_will_play: Hand::Rock,
            i_should_play: Hand::Paper,
        },
        Strategy {
            opponent_will_play: Hand::Paper,
            i_should_play: Hand::Rock,
        },
        Strategy {
            opponent_will_play: Hand::Scissors,
            i_should_play: Hand::Scissors,
        },
    ];

    const TARGET_STRATS: &[Strategy] = &[
        Strategy {
            opponent_will_play: Hand::Rock,
            i_should_play: Hand::Rock,
        },
        Strategy {
            opponent_will_play: Hand::Paper,
            i_should_play: Hand::Rock,
        },
        Strategy {
            opponent_will_play: Hand::Scissors,
            i_should_play: Hand::Rock,
        },
    ];

    #[test]
    fn strategy_guide_is_parsed() {
        assert_equal(
            as_guide(parse_input(EXAMPLE_INPUT).unwrap().iter()),
            EXAMPLE_GUIDE.iter().copied(),
        );
    }

    #[test]
    fn strategy_scores_are_calculated() {
        assert_equal(EXAMPLE_GUIDE.into_iter().map(Strategy::score), [8, 1, 6])
    }

    #[test]
    fn target_strategies_are_found() {
        assert_equal(
            target_strategies(parse_input(EXAMPLE_INPUT).unwrap().iter()),
            TARGET_STRATS.iter().copied(),
        );
    }
}
