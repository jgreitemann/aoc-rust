use std::num::ParseIntError;

use aoc_companion::prelude::*;
use itertools::{iterate, Itertools};
use rayon::iter::{ParallelBridge, ParallelIterator};

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

fn profit(seed: u64, trigger: [i8; 4]) -> u64 {
    prices(seed)
        .zip(
            std::iter::once(0)
                .chain(prices(seed))
                .tuple_windows()
                .map(|(lhs, rhs)| rhs as i8 - lhs as i8),
        )
        .take(2000)
        .tuple_windows()
        .find(|((_, d1), (_, d2), (_, d3), (_, d4))| [*d1, *d2, *d3, *d4] == trigger)
        .map(|w| w.3 .0)
        .unwrap_or(0)
}

fn trigger_combos() -> impl Iterator<Item = [i8; 4]> {
    (-9..=9)
        .cartesian_product(-9..=9)
        .cartesian_product(-9..=9)
        .cartesian_product(-9..=9)
        .map(|(((d1, d2), d3), d4)| [d1, d2, d3, d4])
}

fn most_profit(seeds: &[u64]) -> u64 {
    trigger_combos()
        .par_bridge()
        .map(|trigger| seeds.iter().map(|&seed| profit(seed, trigger)).sum())
        .max()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_TRIGGER: [i8; 4] = [-2, 1, -1, 3];

    #[test]
    fn profit_for_example_trigger() {
        assert_eq!(profit(1, EXAMPLE_TRIGGER), 7);
        assert_eq!(profit(2, EXAMPLE_TRIGGER), 7);
        assert_eq!(profit(3, EXAMPLE_TRIGGER), 0);
        assert_eq!(profit(2024, EXAMPLE_TRIGGER), 9);
    }

    #[test]
    fn most_profit_for_example() {
        assert_eq!(most_profit(&[1, 2, 3, 2024]), 23);
    }
}
