use std::num::ParseIntError;

use aoc_companion::prelude::*;
use aoc_utils::array;

const FACTORS: [u64; 2] = [16807, 48271];
const MODULUS: u64 = 2147483647;

pub struct Door {
    start: [u64; 2],
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("expected two lines, got {0}")]
    WrongNumberOfLines(usize),
    #[error("a line did not start with the expected prefix")]
    MissingPrefix,
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
}

impl ParseInput<'_> for Door {
    type Error = ParseError;

    fn parse(input: &str) -> Result<Self, Self::Error> {
        let lines: [&str; 2] = array::from_iter_exact(input.lines())
            .map_err(|lines| ParseError::WrongNumberOfLines(lines.len()))?;

        let start = array::try_map(lines, |line| {
            let Some(line) = line.strip_prefix("Generator ") else {
                return Err(ParseError::MissingPrefix);
            };
            let line = &line[1..];
            let Some(line) = line.strip_prefix(" starts with ") else {
                return Err(ParseError::MissingPrefix);
            };
            Ok(line.parse()?)
        })?;

        Ok(Door { start })
    }
}

impl Part1 for Door {
    type Output = usize;
    type Error = std::convert::Infallible;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        Ok(count_matching_pairs_part_1(self.start))
    }
}

impl Part2 for Door {
    type Output = usize;
    type Error = std::convert::Infallible;

    fn part2(&self) -> Result<Self::Output, Self::Error> {
        Ok(count_matching_pairs_part_2(self.start))
    }
}

fn generator(mut start: u64, factor: u64) -> impl Iterator<Item = u64> {
    std::iter::from_fn(move || {
        start = (start * factor) % MODULUS;
        Some(start)
    })
}

fn lowest_16_bits_match(lhs: u64, rhs: u64) -> bool {
    const MASK: u64 = (1 << 16) - 1;
    (lhs & MASK) == (rhs & MASK)
}

fn count_matching_pairs_part_1(starts: [u64; 2]) -> usize {
    generator(starts[0], FACTORS[0])
        .zip(generator(starts[1], FACTORS[1]))
        .take(40_000_000)
        .filter(|&(lhs, rhs)| lowest_16_bits_match(lhs, rhs))
        .count()
}

fn count_matching_pairs_part_2(starts: [u64; 2]) -> usize {
    generator(starts[0], FACTORS[0])
        .filter(|x| x % 4 == 0)
        .zip(generator(starts[1], FACTORS[1]).filter(|x| x % 8 == 0))
        .take(5_000_000)
        .filter(|&(lhs, rhs)| lowest_16_bits_match(lhs, rhs))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "Generator A starts with 65
Generator B starts with 8921
";

    const EXAMPLE_STARTS: [u64; 2] = [65, 8921];

    const GENERATOR_A_SEQ: [u64; 5] = [1092455, 1181022009, 245556042, 1744312007, 1352636452];
    const GENERATOR_B_SEQ: [u64; 5] = [430625591, 1233683848, 1431495498, 137874439, 285222916];

    #[test]
    fn parse_input() {
        assert_eq!(Door::parse(EXAMPLE_INPUT).unwrap().start, EXAMPLE_STARTS);
    }

    #[test]
    fn generator_example_sequences() {
        assert_eq!(
            array::from_iter(generator(EXAMPLE_STARTS[0], FACTORS[0])).unwrap(),
            GENERATOR_A_SEQ
        );
        assert_eq!(
            array::from_iter(generator(EXAMPLE_STARTS[1], FACTORS[1])).unwrap(),
            GENERATOR_B_SEQ
        );
    }

    #[test]
    fn check_lowest_16_bits() {
        assert_eq!(
            array::from_iter_exact(
                GENERATOR_A_SEQ
                    .iter()
                    .zip(GENERATOR_B_SEQ.iter())
                    .map(|(&lhs, &rhs)| lowest_16_bits_match(lhs, rhs))
            )
            .unwrap(),
            [false, false, true, false, false]
        );
    }

    #[test]
    fn number_of_matching_pairs_for_part_1_in_example() {
        assert_eq!(count_matching_pairs_part_1(EXAMPLE_STARTS), 588);
    }

    #[test]
    fn number_of_matching_pairs_for_part_2_in_example() {
        assert_eq!(count_matching_pairs_part_2(EXAMPLE_STARTS), 309);
    }
}
