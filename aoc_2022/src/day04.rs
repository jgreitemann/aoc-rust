use aoc_companion::prelude::*;

use itertools::Itertools;
use thiserror::Error;

use std::ops::RangeInclusive;
use std::str::FromStr;

#[derive(Debug)]
pub struct Door {
    pairs: Vec<Pair>,
}

impl ParseInput<'_> for Door {
    type Error = ParseError;

    fn parse(input: &str) -> Result<Self, Self::Error> {
        let pairs = input.lines().map(str::parse).try_collect()?;
        Ok(Door { pairs })
    }
}

impl Part1 for Door {
    type Output = usize;
    type Error = std::convert::Infallible;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        Ok(self.pairs.iter().filter(|&p| p.is_encompassing()).count())
    }
}

impl Part2 for Door {
    type Output = usize;
    type Error = std::convert::Infallible;

    fn part2(&self) -> Result<Self::Output, Self::Error> {
        Ok(self.pairs.iter().filter(|&p| p.is_overlapping()).count())
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Could not find the separator {0:?}")]
    SeparatorNotFound(char),
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
}

#[derive(Debug, Clone, PartialEq)]
struct Assignment(RangeInclusive<u32>);

impl Assignment {
    fn encompasses(&self, other: &Assignment) -> bool {
        self.0.contains(other.0.start()) && self.0.contains(other.0.end())
    }

    fn overlaps_partially(&self, other: &Assignment) -> bool {
        self.0.contains(other.0.start()) || self.0.contains(other.0.end())
    }
}

impl FromStr for Assignment {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (begin, end) = s
            .split_once('-')
            .ok_or(ParseError::SeparatorNotFound('-'))?;
        Ok(Self(
            begin.parse().map_err(ParseError::from)?..=end.parse().map_err(ParseError::from)?,
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Pair(Assignment, Assignment);

impl Pair {
    fn is_encompassing(&self) -> bool {
        self.0.encompasses(&self.1) || self.1.encompasses(&self.0)
    }

    fn is_overlapping(&self) -> bool {
        self.0.overlaps_partially(&self.1) || self.1.overlaps_partially(&self.0)
    }
}

impl FromStr for Pair {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (first, second) = s
            .split_once(',')
            .ok_or(ParseError::SeparatorNotFound(','))?;
        Ok(Pair(first.parse()?, second.parse()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use itertools::assert_equal;

    const EXAMPLE_INPUT: &str = r"2-4,6-8
2-3,4-5
5-7,7-9
2-8,3-7
6-6,4-6
2-6,4-8
3-7,2-8";

    const EXAMPLE_PAIRS: &[Pair] = &[
        Pair(Assignment(2..=4), Assignment(6..=8)),
        Pair(Assignment(2..=3), Assignment(4..=5)),
        Pair(Assignment(5..=7), Assignment(7..=9)),
        Pair(Assignment(2..=8), Assignment(3..=7)),
        Pair(Assignment(6..=6), Assignment(4..=6)),
        Pair(Assignment(2..=6), Assignment(4..=8)),
        Pair(Assignment(3..=7), Assignment(2..=8)),
    ];

    #[test]
    fn valid_input_is_parsed() {
        assert_equal(
            EXAMPLE_INPUT
                .lines()
                .map(|line| line.parse::<Pair>().unwrap()),
            EXAMPLE_PAIRS.iter().cloned(),
        );
    }

    #[test]
    fn invalid_input_produces_errors() {
        assert_matches!(
            "1-2;3-4".parse::<Pair>(),
            Err(ParseError::SeparatorNotFound(','))
        );
        assert_matches!(
            "1-2,3,4".parse::<Pair>(),
            Err(ParseError::SeparatorNotFound('-'))
        );
        assert_matches!(
            "1—2,3-4".parse::<Pair>(),
            Err(ParseError::SeparatorNotFound('-'))
        );
        assert_matches!("NaN-2,3-4".parse::<Pair>(), Err(ParseError::ParseInt(_)));
        assert_matches!("1-2-3,3-4".parse::<Pair>(), Err(ParseError::ParseInt(_)));
        assert_matches!("1-2,3.0-4".parse::<Pair>(), Err(ParseError::ParseInt(_)));
        assert_matches!("1-2,3-∞".parse::<Pair>(), Err(ParseError::ParseInt(_)));
    }

    #[test]
    fn fully_contained_pairs_are_identified() {
        assert_equal(
            EXAMPLE_PAIRS.iter().map(Pair::is_encompassing),
            [false, false, false, true, true, false, true],
        );
    }

    #[test]
    fn overlapping_pairs_are_identified() {
        assert_equal(
            EXAMPLE_PAIRS.iter().map(Pair::is_overlapping),
            [false, false, true, true, true, true, true],
        );
    }
}
