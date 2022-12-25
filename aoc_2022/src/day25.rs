use aoc_companion::prelude::*;

use itertools::Itertools;
use thiserror::Error;

use std::str::FromStr;

pub struct Door {
    fuel: Vec<Snafu>,
}

impl ParseInput<'_> for Door {
    type Error = ParseError;

    fn parse(input: &str) -> Result<Self, Self::Error> {
        input
            .lines()
            .map(str::parse)
            .try_collect()
            .map(|fuel| Self { fuel })
    }
}

impl Part1 for Door {
    type Output = Snafu;
    type Error = std::convert::Infallible;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        Ok(sum_snafu_numbers(&self.fuel))
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseError {
    #[error("Encountered invalid SNAFU digit")]
    InvalidSnafuDigit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snafu(Vec<i64>);

impl FromStr for Snafu {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.as_bytes()
            .iter()
            .rev()
            .map(|d| match d {
                b'=' => Ok(-2),
                b'-' => Ok(-1),
                b'0' => Ok(0),
                b'1' => Ok(1),
                b'2' => Ok(2),
                _ => Err(ParseError::InvalidSnafuDigit),
            })
            .try_collect()
            .map(|digits| Self(digits))
    }
}

impl std::fmt::Display for Snafu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.iter().rev().fold(Ok(()), |res, digit| {
            res.and_then(|_| {
                f.write_str(match digit {
                    -2 => "=",
                    -1 => "-",
                    0 => "0",
                    1 => "1",
                    2 => "2",
                    _ => "#",
                })
            })
        })
    }
}

impl From<Snafu> for i64 {
    fn from(value: Snafu) -> Self {
        value
            .0
            .into_iter()
            .fold((1, 0), |(base, sum), digit| (base * 5, sum + base * digit))
            .1
    }
}

impl From<i64> for Snafu {
    fn from(mut value: i64) -> Self {
        let mut digits = Vec::new();
        let mut carry = 0;
        while value > 0 || carry > 0 {
            let mut x = value % 5 + std::mem::replace(&mut carry, 0);
            value /= 5;
            while x >= 3 {
                x -= 5;
                carry += 1;
            }
            digits.push(x as i64);
        }
        Self(digits)
    }
}

fn sum_snafu_numbers(numbers: &[Snafu]) -> Snafu {
    numbers.iter().cloned().map(i64::from).sum::<i64>().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DECIMAL_NUMBERS: [i64; 13] = [1747, 906, 198, 11, 201, 31, 1257, 32, 353, 107, 7, 3, 37];

    fn snafu_numbers() -> [Snafu; 13] {
        [
            "1=-0-2".parse().unwrap(),
            "12111".parse().unwrap(),
            "2=0=".parse().unwrap(),
            "21".parse().unwrap(),
            "2=01".parse().unwrap(),
            "111".parse().unwrap(),
            "20012".parse().unwrap(),
            "112".parse().unwrap(),
            "1=-1=".parse().unwrap(),
            "1-12".parse().unwrap(),
            "12".parse().unwrap(),
            "1=".parse().unwrap(),
            "122".parse().unwrap(),
        ]
    }

    #[test]
    fn snafu_numbers_can_be_parsed() {
        assert_eq!(
            "1121-1110-1=0".parse::<Snafu>(),
            Ok(Snafu(vec![0, -2, 1, -1, 0, 1, 1, 1, -1, 1, 2, 1, 1]))
        );
        assert_eq!(
            "1121-1x10-1=0".parse::<Snafu>(),
            Err(ParseError::InvalidSnafuDigit)
        );
    }

    #[test]
    fn snafu_numbers_can_be_formatted() {
        assert_eq!(
            &format!("{}", Snafu(vec![0, -2, 1, -1, 0, 1, 1, 1, -1, 1, 2, 1, 1])),
            "1121-1110-1=0"
        );
    }

    #[test]
    fn snafu_can_be_converted_to_decimal() {
        assert_eq!(snafu_numbers().map(i64::from), DECIMAL_NUMBERS);
    }

    #[test]
    fn decimal_can_be_converted_to_snafu() {
        assert_eq!(DECIMAL_NUMBERS.map(Snafu::from), snafu_numbers());
    }

    #[test]
    fn snafu_numbers_can_be_summed_up() {
        assert_eq!(
            sum_snafu_numbers(&snafu_numbers()),
            "2=-1=0".parse().unwrap()
        );
    }
}
