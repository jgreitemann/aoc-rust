use std::num::ParseIntError;

use aoc_companion::prelude::*;
use aoc_utils::cache::cached;
use itertools::Itertools;

pub(crate) struct Door {
    start: Vec<u64>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseIntError> {
        Ok(Door {
            start: parse_input(input)?,
        })
    }

    fn part1(&self) -> usize {
        blink_seq(&self.start).nth(25).unwrap().len()
    }

    fn part2(&self) -> usize {
        stone_count(75, &self.start)
    }
}

fn parse_input(input: &str) -> Result<Vec<u64>, ParseIntError> {
    input.split_ascii_whitespace().map(str::parse).try_collect()
}

fn try_split(r: u64) -> Result<(u64, u64), u64> {
    if r == 0 {
        return Err(0);
    }
    let digits = r.ilog10() + 1;
    if digits % 2 == 0 {
        let half_pow = 10u64.pow(digits / 2);
        Ok((r / half_pow, r % half_pow))
    } else {
        Err(r)
    }
}

fn blink(rocks: &[u64]) -> Vec<u64> {
    rocks
        .iter()
        .flat_map(|r| match try_split(*r) {
            Ok((lhs, rhs)) => [Some(lhs), Some(rhs)],
            Err(0) => [Some(1), None],
            Err(x) => [Some(x * 2024), None],
        })
        .flatten()
        .collect()
}

fn blink_seq(rocks: &[u64]) -> impl Iterator<Item = Vec<u64>> {
    itertools::iterate(rocks.to_vec(), |rocks| blink(rocks.as_slice()))
}

fn stone_count(n: usize, rocks: &[u64]) -> usize {
    let mut cached_stone_count_recursive = cached(stone_count_recursive);
    rocks
        .iter()
        .map(move |&r| cached_stone_count_recursive((n, r)))
        .sum()
}

fn stone_count_recursive(
    (n, r): (usize, u64),
    recurse: &mut dyn FnMut((usize, u64)) -> usize,
) -> usize {
    if n == 0 {
        1
    } else {
        match try_split(r) {
            Ok((lhs, rhs)) => recurse((n - 1, lhs)) + recurse((n - 1, rhs)),
            Err(0) => recurse((n - 1, 1)),
            Err(x) => recurse((n - 1, 2024 * x)),
        }
    }
}

#[cfg(test)]
mod tests {
    use itertools::assert_equal;

    use super::*;

    const EXAMPLE_INPUT: &str = "0 1 10 99 999";
    const EXAMPLE_STONES: &[u64] = &[0, 1, 10, 99, 999];
    const EXAMPLE_START: &[u64] = &[125, 17];

    #[test]
    fn parse_example() {
        assert_eq!(parse_input(EXAMPLE_INPUT).unwrap(), EXAMPLE_STONES);
    }

    #[test]
    fn blinking_repeatedly() {
        assert_equal(
            blink_seq(EXAMPLE_START).take(7),
            [
                vec![125, 17],
                vec![253000, 1, 7],
                vec![253, 0, 2024, 14168],
                vec![512072, 1, 20, 24, 28676032],
                vec![512, 72, 2024, 2, 0, 2, 4, 2867, 6032],
                vec![1036288, 7, 2, 20, 24, 4048, 1, 4048, 8096, 28, 67, 60, 32],
                vec![
                    2097446912, 14168, 4048, 2, 0, 2, 4, 40, 48, 2024, 40, 48, 80, 96, 2, 8, 6, 7,
                    6, 0, 3, 2,
                ],
            ],
        );
    }

    #[test]
    fn stones_after_25_blinks() {
        assert_eq!(blink_seq(EXAMPLE_START).nth(25).unwrap().len(), 55312);
    }

    #[test]
    fn stone_count_after_25_blinks() {
        assert_eq!(stone_count(25, EXAMPLE_START), 55312);
    }
}
