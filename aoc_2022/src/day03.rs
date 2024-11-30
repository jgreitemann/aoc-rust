use aoc_companion::prelude::*;

use itertools::Itertools;
use thiserror::Error;

use std::collections::HashSet;

pub(crate) struct Door {
    rucksacks: Vec<Vec<u32>>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseError> {
        Ok(Self {
            rucksacks: input.lines().map(parse_priorities).try_collect()?,
        })
    }

    fn part1(&self) -> Result<u32, Error> {
        self.rucksacks
            .iter()
            .map(|rucksack| Rucksack::new(rucksack).common_item_priority())
            .fold_ok(0, std::ops::Add::add)
    }

    fn part2(&self) -> Result<u32, Error> {
        group_batch_priorities(&self.rucksacks).map(|batches| batches.iter().sum())
    }
}

#[derive(Debug, Error)]
pub(crate) enum ParseError {
    #[error("Cannot assign a priority to item '{0}'")]
    InvalidItemChar(char),
}

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("There no one common item")]
    NoUniqueCommonItem,
    #[error("The group is empty")]
    EmptyGroup,
}

#[derive(Debug)]
struct Rucksack {
    first_compartment: HashSet<u32>,
    second_compartment: HashSet<u32>,
}

impl Rucksack {
    fn new(prios: &[u32]) -> Self {
        let (first, second) = prios.split_at(prios.len() / 2);
        Self {
            first_compartment: HashSet::from_iter(first.iter().copied()),
            second_compartment: HashSet::from_iter(second.iter().copied()),
        }
    }

    fn common_item_priority(&self) -> Result<u32, Error> {
        HashSet::intersection(&self.first_compartment, &self.second_compartment)
            .copied()
            .exactly_one()
            .map_err(|_| Error::NoUniqueCommonItem)
    }
}

fn parse_priorities(line: &str) -> Result<Vec<u32>, ParseError> {
    line.chars().map(priority).collect()
}

fn priority(c: char) -> Result<u32, ParseError> {
    match c {
        _ if c.is_ascii_lowercase() => Ok(c as u32 - 'a' as u32 + 1),
        _ if c.is_ascii_uppercase() => Ok(c as u32 - 'A' as u32 + 27),
        _ => Err(ParseError::InvalidItemChar(c)),
    }
}

fn batch_priority<'a>(group: impl Iterator<Item = &'a [u32]>) -> Result<u32, Error> {
    group
        .map(|slice| HashSet::from_iter(slice.iter().copied()))
        .reduce(|lhs: HashSet<u32>, rhs| HashSet::intersection(&lhs, &rhs).copied().collect())
        .ok_or(Error::EmptyGroup)?
        .into_iter()
        .exactly_one()
        .map_err(|_| Error::NoUniqueCommonItem)
}

fn group_batch_priorities<R: AsRef<[u32]>>(rucksacks: &[R]) -> Result<Vec<u32>, Error> {
    rucksacks
        .iter()
        .map(AsRef::as_ref)
        .chunks(3)
        .into_iter()
        .map(batch_priority)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use rstest::rstest;

    #[rstest]
    #[case("vJrwpWtwJgWrhcsFMMfFFhFp", 16)]
    #[case("jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL", 38)]
    #[case("PmmdzqPrVvPwwTWBwg", 42)]
    #[case("wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn", 22)]
    #[case("ttgJtRGJQctTZtZT", 20)]
    #[case("CrZsJsPPZsGzwwsLwLmpwMDw", 19)]
    fn common_item_priority(#[case] s: &str, #[case] common: u32) {
        assert_eq!(
            Rucksack::new(&parse_priorities(s).unwrap())
                .common_item_priority()
                .unwrap(),
            common
        );
    }

    #[test]
    fn no_unique_common_item() {
        assert_matches!(
            Rucksack::new(&parse_priorities("abcd").unwrap()).common_item_priority(),
            Err(Error::NoUniqueCommonItem)
        );
        assert_matches!(
            Rucksack::new(&parse_priorities("abab").unwrap()).common_item_priority(),
            Err(Error::NoUniqueCommonItem)
        );
    }

    #[test]
    fn group_batches_are_found() {
        let prios = [
            "vJrwpWtwJgWrhcsFMMfFFhFp",
            "jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL",
            "PmmdzqPrVvPwwTWBwg",
            "wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn",
            "ttgJtRGJQctTZtZT",
            "CrZsJsPPZsGzwwsLwLmpwMDw",
        ]
        .map(|line| parse_priorities(line).unwrap());

        assert_eq!(group_batch_priorities(&prios).unwrap(), [18, 52]);
    }
}
