use aoc_companion::prelude::*;

use itertools::Itertools;
use std::num::ParseIntError;

#[derive(Debug)]
pub struct Door {
    spreadsheet: Vec<Vec<i32>>,
}

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("spreadsheet row is empty")]
    RowIsEmpty,
    #[error("row does not contain a pair of evenly divisible numbers")]
    RowDoesNotContainEvenlyDivisibleNumbers,
}

impl ParseInput for Door {
    type Error = ParseIntError;

    fn parse(input: &str) -> Result<Self, Self::Error> {
        let spreadsheet = input
            .lines()
            .map(|line| line.split_whitespace().map(|word| word.parse()).collect())
            .collect::<Result<_, ParseIntError>>()?;
        Ok(Self { spreadsheet })
    }
}

impl Part1 for Door {
    type Output = i32;
    type Error = Error;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        checksum(&self.spreadsheet, minmax_diff)
    }
}

impl Part2 for Door {
    type Output = i32;
    type Error = Error;

    fn part2(&self) -> Result<Self::Output, Self::Error> {
        checksum(&self.spreadsheet, evenly_divisible_quotient)
    }
}

fn checksum<R, F>(spreadsheet: &[R], row_fn: F) -> Result<i32, Error>
where
    R: AsRef<[i32]>,
    F: Fn(&[i32]) -> Result<i32, Error>,
{
    spreadsheet.iter().map(R::as_ref).map(row_fn).sum()
}

fn minmax_diff(row: &[i32]) -> Result<i32, Error> {
    use itertools::MinMaxResult::*;
    match row.iter().minmax() {
        MinMax(min, max) => Ok(*max - *min),
        OneElement(_) => Ok(0),
        NoElements => Err(Error::RowIsEmpty),
    }
}

fn evenly_divisible_quotient(row: &[i32]) -> Result<i32, Error> {
    row.iter()
        .tuple_combinations()
        .filter_map(|pair| match pair {
            (a, b) if a % b == 0 => Some(a / b),
            (a, b) if b % a == 0 => Some(b / a),
            _ => None,
        })
        .next()
        .ok_or(Error::RowDoesNotContainEvenlyDivisibleNumbers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    const EXAMPLE_WITH_SINGLE_SPACES: &str = r#"5 1 9 5
7 5 3
2 4 6 8"#;
    const EXAMPLE_WITH_MULTIPLE_SPACES: &str = r#"5  1   9  5
7  5 3
2  4   6  8"#;
    const EXAMPLE_SPREADSHEET: &[&[i32]] = &[&[5, 1, 9, 5], &[7, 5, 3], &[2, 4, 6, 8]];
    const DIVISION_SPREADSHEET: &[&[i32]] = &[&[5, 9, 2, 8], &[9, 4, 7, 3], &[3, 8, 6, 5]];
    const PRIME_SPREADSHEET: &[&[i32]] = &[&[2, 3, 5, 7, 11]];

    #[test]
    fn spreadsheet_is_parsed() {
        assert_eq!(
            Door::parse(EXAMPLE_WITH_SINGLE_SPACES).unwrap().spreadsheet,
            EXAMPLE_SPREADSHEET
        );
        assert_eq!(
            Door::parse(EXAMPLE_WITH_MULTIPLE_SPACES)
                .unwrap()
                .spreadsheet,
            EXAMPLE_SPREADSHEET
        );
    }

    #[test]
    fn spreadsheet_fails_to_parse_non_numeric_input() {
        assert_matches!(Door::parse("1 2 three 4"), Err(ParseIntError { .. }));
    }

    #[test]
    fn spreadsheet_minmax_checksum() {
        assert_eq!(checksum(EXAMPLE_SPREADSHEET, minmax_diff), Ok(18));
    }

    #[test]
    fn spreadsheet_quotient_checksum() {
        assert_eq!(
            checksum(DIVISION_SPREADSHEET, evenly_divisible_quotient),
            Ok(9)
        );
    }

    #[test]
    fn row_of_primes_yields_an_error_for_quotient_checksum() {
        assert_eq!(
            checksum(PRIME_SPREADSHEET, evenly_divisible_quotient),
            Err(Error::RowDoesNotContainEvenlyDivisibleNumbers)
        );
    }
}
