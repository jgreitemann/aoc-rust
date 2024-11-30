use aoc_companion::prelude::*;
use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    num::ParseIntError,
};

pub(crate) struct Door {
    bank: Vec<i32>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseIntError> {
        let bank = input
            .split_whitespace()
            .map(|s| s.parse())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { bank })
    }

    fn part1(&self) -> usize {
        count_redistribution_cycles_until_recurrence(self.bank.clone())
    }

    fn part2(&self) -> usize {
        redistribution_cycle_loop_length(self.bank.clone())
    }
}

fn redistribute(bank: &mut [i32]) {
    if let Some(pos) = bank.iter().position_min_by_key(|&x| -x) {
        let budget = std::mem::replace(&mut bank[pos], 0);
        let total_buckets = bank.len() as i32;
        let (head, tail) = bank.split_at_mut(pos + 1);
        tail.iter_mut().chain(head.iter_mut()).fold(
            (budget, total_buckets),
            |(remaining_budget, remaining_buckets), bucket| {
                let div = remaining_budget / remaining_buckets;
                let rem = remaining_budget % remaining_buckets;
                let spend = if rem == 0 { div } else { div + 1 };
                *bucket += spend;
                (remaining_budget - spend, remaining_buckets - 1)
            },
        );
    }
}

struct Redistributor {
    current: Vec<i32>,
}

impl Iterator for Redistributor {
    type Item = Vec<i32>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.current.clone();
        redistribute(&mut self.current);
        Some(result)
    }
}

fn count_redistribution_cycles_until_recurrence(bank: Vec<i32>) -> usize {
    Redistributor { current: bank }
        .scan(HashSet::new(), |seen, state| {
            if seen.insert(state) {
                Some(())
            } else {
                None
            }
        })
        .count()
}

fn redistribution_cycle_loop_length(bank: Vec<i32>) -> usize {
    Redistributor { current: bank }
        .enumerate()
        .scan(HashMap::new(), |seen, (idx, state)| {
            if let Some(prev_idx) = seen.insert(state, idx) {
                Some(idx - prev_idx)
            } else {
                Some(0)
            }
        })
        .find(|&x| x > 0)
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redistribution_reproduces_example() {
        let mut bank = [0, 2, 7, 0];
        redistribute(&mut bank);
        assert_eq!(bank, [2, 4, 1, 2]);
        redistribute(&mut bank);
        assert_eq!(bank, [3, 1, 2, 3]);
        redistribute(&mut bank);
        assert_eq!(bank, [0, 2, 3, 4]);
        redistribute(&mut bank);
        assert_eq!(bank, [1, 3, 4, 1]);
        redistribute(&mut bank);
        assert_eq!(bank, [2, 4, 1, 2]);
    }

    #[test]
    fn number_of_redistribution_cycles_until_recurrence_matches_example() {
        assert_eq!(
            count_redistribution_cycles_until_recurrence(vec![0, 2, 7, 0]),
            5
        );
    }

    #[test]
    fn loop_length_of_redistribution_cycles_matches_example() {
        assert_eq!(redistribution_cycle_loop_length(vec![0, 2, 7, 0]), 4);
    }
}
