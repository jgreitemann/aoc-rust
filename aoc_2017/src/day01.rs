use aoc_companion::prelude::*;
use itertools::Itertools;

pub(crate) struct Door {
    digits: Vec<u32>,
}

#[derive(Debug, Error, PartialEq)]
pub(crate) enum Error {
    #[error("Encountered an input character which is not a digit: {0:?}")]
    CharacterIsNotDigit(char),
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, Error> {
        Ok(Self {
            digits: digits(input)?,
        })
    }

    fn part1(&self) -> u32 {
        adjacent_equal_digit_sum(&self.digits)
    }

    fn part2(&self) -> u32 {
        opposing_equal_digit_sum(&self.digits)
    }
}

fn digits(input: &str) -> Result<Vec<u32>, Error> {
    input
        .chars()
        .map(|c| c.to_digit(10).ok_or(Error::CharacterIsNotDigit(c)))
        .collect()
}

fn adjacent_equal_digit_sum(digits: &[u32]) -> u32 {
    digits
        .iter()
        .circular_tuple_windows()
        .map(|(a, b)| if a == b { *a } else { 0 })
        .sum()
}

fn opposing_equal_digit_sum(digits: &[u32]) -> u32 {
    let opposed_digits = digits
        .iter()
        .cycle()
        .skip(digits.len() / 2)
        .take(digits.len());
    digits
        .iter()
        .zip_eq(opposed_digits)
        .map(|(a, b)| if a == b { *a } else { 0 })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digits_are_parsed() {
        assert_eq!(digits(""), Ok(vec![]));
        assert_eq!(digits("7"), Ok(vec![7]));
        assert_eq!(digits("1234"), Ok(vec![1, 2, 3, 4]));
        assert_eq!(digits("12E4"), Err(Error::CharacterIsNotDigit('E')));
    }

    #[test]
    fn example_adjacent_equal_digit_sums() {
        assert_eq!(adjacent_equal_digit_sum(&[1, 1, 2, 2]), 3);
        assert_eq!(adjacent_equal_digit_sum(&[1, 1, 1, 1]), 4);
        assert_eq!(adjacent_equal_digit_sum(&[1, 2, 3, 4]), 0);
        assert_eq!(adjacent_equal_digit_sum(&[9, 1, 2, 1, 2, 1, 2, 9]), 9);
    }

    #[test]
    fn example_opposing_equal_digit_sums() {
        assert_eq!(opposing_equal_digit_sum(&[1, 2, 1, 2]), 6);
        assert_eq!(opposing_equal_digit_sum(&[1, 2, 2, 1]), 0);
        assert_eq!(opposing_equal_digit_sum(&[1, 2, 3, 4, 2, 5]), 4);
        assert_eq!(opposing_equal_digit_sum(&[1, 2, 3, 1, 2, 3]), 12);
        assert_eq!(opposing_equal_digit_sum(&[1, 2, 1, 3, 1, 4, 1, 5]), 4);
    }
}
