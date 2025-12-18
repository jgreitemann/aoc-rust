use std::{collections::HashMap, num::ParseIntError};

use aoc_companion::prelude::*;
use itertools::Itertools as _;

pub(crate) struct Door {
    starting_numbers: Vec<usize>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseIntError> {
        input
            .split(',')
            .map(str::parse)
            .try_collect()
            .map(|starting_numbers| Door { starting_numbers })
    }

    fn part1(&self) -> usize {
        Game::from(self.starting_numbers.iter().copied())
            .nth(2019)
            .unwrap()
    }

    fn part2(&self) -> usize {
        Game::from(self.starting_numbers.iter().copied())
            .nth(29_999_999)
            .unwrap()
    }
}

struct Game<I: Iterator<Item = usize>> {
    starting_numbers: I,
    next: Option<usize>,
    n: usize,
    last_seen: HashMap<usize, usize>,
}

impl<I: Iterator<Item = usize>> Game<I> {
    fn from(starting_numbers: impl IntoIterator<Item = usize, IntoIter = I>) -> Self {
        Self {
            starting_numbers: starting_numbers.into_iter(),
            next: None,
            n: 0,
            last_seen: HashMap::new(),
        }
    }
}

impl<I: Iterator<Item = usize>> Iterator for Game<I> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.starting_numbers.next().or_else(|| self.next.take())?;
        self.n += 1;
        if let Some(before) = self.last_seen.insert(current, self.n) {
            self.next = Some(self.n - before);
        } else {
            self.next = Some(0);
        }
        Some(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_game_number_sequence() {
        itertools::assert_equal(
            Game::from([0, 3, 6]).take(10),
            [0, 3, 6, 0, 3, 3, 1, 0, 4, 0],
        );
    }

    #[test]
    fn correct_2020th_number() {
        assert_eq!(Game::from([0, 3, 6]).nth(2019), Some(436));
        assert_eq!(Game::from([1, 3, 2]).nth(2019), Some(1));
        assert_eq!(Game::from([2, 1, 3]).nth(2019), Some(10));
        assert_eq!(Game::from([1, 2, 3]).nth(2019), Some(27));
        assert_eq!(Game::from([2, 3, 1]).nth(2019), Some(78));
        assert_eq!(Game::from([3, 2, 1]).nth(2019), Some(438));
        assert_eq!(Game::from([3, 1, 2]).nth(2019), Some(1836));
    }

    #[test]
    #[ignore = "slow"]
    fn correct_30_000_000th_number() {
        assert_eq!(Game::from([0, 3, 6]).nth(29_999_999), Some(175594));
        assert_eq!(Game::from([1, 3, 2]).nth(29_999_999), Some(2578));
        assert_eq!(Game::from([2, 1, 3]).nth(29_999_999), Some(3544142));
        assert_eq!(Game::from([1, 2, 3]).nth(29_999_999), Some(261214));
        assert_eq!(Game::from([2, 3, 1]).nth(29_999_999), Some(6895259));
        assert_eq!(Game::from([3, 2, 1]).nth(29_999_999), Some(18));
        assert_eq!(Game::from([3, 1, 2]).nth(29_999_999), Some(362));
    }
}
