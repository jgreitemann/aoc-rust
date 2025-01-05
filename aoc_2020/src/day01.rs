use std::num::ParseIntError;

use anyhow::anyhow;
use aoc_companion::prelude::*;
use itertools::Itertools;

pub(crate) struct Door(Vec<i32>);

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseIntError> {
        input
            .split_whitespace()
            .map(str::parse)
            .try_collect()
            .map(Door)
    }

    fn part1(&self) -> Result<i32> {
        entries_which_sum_to_2020(&self.0)
            .ok_or_else(|| anyhow!("no two entries sum to 2020"))
            .map(|[x, y]| x * y)
    }

    fn part2(&self) -> Result<i32> {
        entries_which_sum_to_2020(&self.0)
            .ok_or_else(|| anyhow!("no three entries sum to 2020"))
            .map(|[x, y, z]| x * y * z)
    }
}

fn entries_which_sum_to_2020<const N: usize>(entries: &[i32]) -> Option<[i32; N]> {
    entries
        .iter()
        .cloned()
        .array_combinations::<N>()
        .find(|array| array.iter().sum::<i32>() == 2020)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_ENTRIES: &[i32] = &[1721, 979, 366, 299, 675, 1456];

    #[test]
    fn two_entries_which_sum_to_2020() {
        assert_eq!(
            entries_which_sum_to_2020(EXAMPLE_ENTRIES).unwrap(),
            [1721, 299]
        );
    }

    #[test]
    fn three_entries_which_sum_to_2020() {
        assert_eq!(
            entries_which_sum_to_2020(EXAMPLE_ENTRIES).unwrap(),
            [979, 366, 675]
        );
    }
}
