use std::ops::RangeInclusive;

use aoc_companion::prelude::*;
use itertools::Itertools as _;
use num_traits::Euclid as _;

pub(crate) struct Door {
    ranges: Vec<RangeInclusive<u64>>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Door> {
        parse_ranges(input).map(|ranges| Door { ranges })
    }

    fn part1(&self) -> u64 {
        self.ranges
            .iter()
            .cloned()
            .flatten()
            .filter(is_simple_invalid_id)
            .sum()
    }

    fn part2(&self) -> u64 {
        self.ranges
            .iter()
            .cloned()
            .flatten()
            .filter(is_invalid_id)
            .sum()
    }
}

fn parse_ranges(s: &str) -> Result<Vec<RangeInclusive<u64>>> {
    s.split(',')
        .map(|r| {
            let Some((lhs, rhs)) = r.split_once('-') else {
                anyhow::bail!("missing '-' in range {r:?}");
            };
            Ok(lhs.parse()?..=rhs.parse()?)
        })
        .try_collect()
}

fn is_simple_invalid_id(n: &u64) -> bool {
    let s = n.to_string();
    let (mid, 0) = s.len().div_rem_euclid(&2) else {
        return false;
    };
    let (lhs, rhs) = s.split_at(mid);
    lhs == rhs
}

fn is_invalid_id(n: &u64) -> bool {
    let s = n.to_string().into_bytes();

    (1..=s.len() / 2)
        .filter(|c| s.len().is_multiple_of(*c))
        .any(|chunk_size| s.chunks(chunk_size).all_equal())
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "11-22,95-115,998-1012,1188511880-1188511890,222220-222224,1698522-1698528,446443-446449,38593856-38593862,565653-565659,824824821-824824827,2121212118-2121212124";

    const EXAMPLE_RANGES: &[RangeInclusive<u64>] = &[
        11..=22,
        95..=115,
        998..=1012,
        1188511880..=1188511890,
        222220..=222224,
        1698522..=1698528,
        446443..=446449,
        38593856..=38593862,
        565653..=565659,
        824824821..=824824827,
        2121212118..=2121212124,
    ];

    #[test]
    fn parse_ranges_in_example_input() {
        assert_eq!(parse_ranges(EXAMPLE_INPUT).unwrap(), EXAMPLE_RANGES);
    }

    #[test]
    fn find_simple_invalid_ids_in_ranges() {
        itertools::assert_equal(
            EXAMPLE_RANGES
                .iter()
                .cloned()
                .flatten()
                .filter(is_simple_invalid_id),
            [11, 22, 99, 1010, 1188511885, 222222, 446446, 38593859],
        );
    }

    #[test]
    fn find_all_invalid_ids_in_ranges() {
        itertools::assert_equal(
            EXAMPLE_RANGES
                .iter()
                .cloned()
                .flatten()
                .filter(is_invalid_id),
            [
                11, 22, 99, 111, 999, 1010, 1188511885, 222222, 446446, 38593859, 565656,
                824824824, 2121212121,
            ],
        );
    }
}
