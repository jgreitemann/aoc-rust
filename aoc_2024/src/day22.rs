use std::{collections::HashMap, num::ParseIntError};

use aoc_companion::prelude::*;
use itertools::{iterate, Itertools};

pub(crate) struct Door {
    initial_numbers: Vec<u64>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseIntError> {
        let initial_numbers = input.lines().map(str::parse).try_collect()?;
        Ok(Door { initial_numbers })
    }

    fn part1(&self) -> u64 {
        self.initial_numbers
            .iter()
            .map(|&seed| prng(seed).nth(2000).unwrap())
            .sum()
    }

    fn part2(&self) -> u64 {
        most_profit(&self.initial_numbers)
    }
}

const BIT_MASK_24: u64 = (1 << 24) - 1;

fn prng(seed: u64) -> impl Iterator<Item = u64> {
    iterate(seed, |state| {
        let mut new_state = *state;
        new_state ^= new_state << 6;
        new_state &= BIT_MASK_24;
        new_state ^= new_state >> 5;
        new_state &= BIT_MASK_24;
        new_state ^= new_state << 11;
        new_state &= BIT_MASK_24;
        new_state
    })
}

fn prices(seed: u64) -> impl Iterator<Item = u64> {
    prng(seed).map(|p| p % 10)
}

fn profit_by_diffs(seed: u64) -> HashMap<[i8; 4], u64> {
    prices(seed)
        .zip(
            std::iter::once(0)
                .chain(prices(seed))
                .tuple_windows()
                .map(|(lhs, rhs)| rhs as i8 - lhs as i8),
        )
        .take(2000)
        .tuple_windows()
        .fold(
            HashMap::new(),
            |mut mapping, ((_, d1), (_, d2), (_, d3), (price, d4))| {
                mapping.entry([d1, d2, d3, d4]).or_insert(price);
                mapping
            },
        )
}

fn most_profit(seeds: &[u64]) -> u64 {
    let global_mapping = seeds
        .iter()
        .map(|&seed| profit_by_diffs(seed))
        .reduce(|mut acc, seed_mapping| {
            for (seq, price) in seed_mapping {
                *acc.entry(seq).or_default() += price;
            }
            acc
        })
        .unwrap_or_default();

    global_mapping.values().copied().max().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn most_profit_for_example() {
        assert_eq!(most_profit(&[1, 2, 3, 2024]), 23);
    }
}
