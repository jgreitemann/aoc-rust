use std::{cmp::Ordering, collections::HashSet, num::ParseIntError, str::FromStr};

use aoc_companion::prelude::*;
use itertools::Itertools;

pub(crate) struct Door {
    rules: HashSet<(u32, u32)>,
    updates: Vec<Vec<u32>>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ParseError {
    #[error("The \"\\n\\n\" separating rules from updates is missing")]
    MissingDoubleNewLine,
    #[error("The pipe separating two pages in an ordering rule is missing")]
    MissingPipe,
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseError> {
        let (rules_input, updates_input) = input
            .split_once("\n\n")
            .ok_or(ParseError::MissingDoubleNewLine)?;
        let rules = rules_input
            .lines()
            .map(|line| -> Result<(u32, u32), ParseError> {
                let (lhs, rhs) = line.split_once('|').ok_or(ParseError::MissingPipe)?;
                Ok((lhs.parse()?, rhs.parse()?))
            })
            .try_collect()?;

        let updates = updates_input
            .lines()
            .map(|line| line.split(',').map(u32::from_str).try_collect())
            .try_collect()?;

        Ok(Door { rules, updates })
    }

    fn part1(&self) -> u32 {
        mid_elem_sum(self.sorted_updates())
    }

    fn part2(&self) -> u32 {
        mid_elem_sum(self.restored_updates())
    }
}

impl Door {
    fn rules_cmp(&self) -> impl Fn(&u32, &u32) -> Ordering + Copy + use<'_> {
        |&lhs, &rhs| {
            if lhs == rhs {
                Ordering::Equal
            } else if self.rules.contains(&(lhs, rhs)) {
                Ordering::Less
            } else if self.rules.contains(&(rhs, lhs)) {
                Ordering::Greater
            } else {
                panic!(
                    "Rules don't impose total ordering: unknown relation between {lhs} and {rhs}"
                );
            }
        }
    }

    fn sorted_updates(&self) -> impl Iterator<Item = impl AsRef<[u32]> + use<'_>> {
        let cmp = self.rules_cmp();
        self.updates
            .iter()
            .filter(move |update| update.is_sorted_by(move |lhs, rhs| cmp(lhs, rhs).is_le()))
    }

    fn restored_updates(&self) -> impl Iterator<Item = impl AsRef<[u32]> + use<'_>> {
        self.updates.iter().filter_map(|update| {
            let sorted = update
                .iter()
                .copied()
                .sorted_by(self.rules_cmp())
                .collect_vec();
            (&sorted != update).then_some(sorted)
        })
    }
}

fn mid_elem(seq: &[u32]) -> u32 {
    seq[seq.len() / 2]
}

fn mid_elem_sum(updates: impl Iterator<Item = impl AsRef<[u32]>>) -> u32 {
    updates.map(|u| mid_elem(u.as_ref())).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "\
47|53
97|13
97|61
97|47
75|29
61|13
75|53
29|13
97|29
53|29
61|53
97|53
61|29
47|13
75|47
97|75
47|61
75|61
47|29
75|13
53|13

75,47,61,53,29
97,61,53,29,13
75,29,13
75,97,47,61,53
61,13,29
97,13,75,29,47";

    const EXAMPLE_RULES: &[(u32, u32)] = &[
        (47, 53),
        (97, 13),
        (97, 61),
        (97, 47),
        (75, 29),
        (61, 13),
        (75, 53),
        (29, 13),
        (97, 29),
        (53, 29),
        (61, 53),
        (97, 53),
        (61, 29),
        (47, 13),
        (75, 47),
        (97, 75),
        (47, 61),
        (75, 61),
        (47, 29),
        (75, 13),
        (53, 13),
    ];

    const EXAMPLE_UPDATES: &[&[u32]] = &[
        &[75, 47, 61, 53, 29],
        &[97, 61, 53, 29, 13],
        &[75, 29, 13],
        &[75, 97, 47, 61, 53],
        &[61, 13, 29],
        &[97, 13, 75, 29, 47],
    ];

    #[test]
    fn parse_example_input() {
        let door = Door::parse(EXAMPLE_INPUT).unwrap();
        assert_eq!(
            door.rules,
            HashSet::from_iter(EXAMPLE_RULES.iter().copied())
        );
        assert_eq!(door.updates, EXAMPLE_UPDATES);
    }

    #[test]
    fn mid_elem_sum_over_sorted_example_updates() {
        let door = Door {
            rules: HashSet::from_iter(EXAMPLE_RULES.iter().copied()),
            updates: EXAMPLE_UPDATES.iter().map(|&u| u.into()).collect(),
        };
        assert_eq!(mid_elem_sum(door.sorted_updates()), 143);
    }

    #[test]
    fn mid_elem_sum_over_restored_example_updates() {
        let door = Door {
            rules: HashSet::from_iter(EXAMPLE_RULES.iter().copied()),
            updates: EXAMPLE_UPDATES.iter().map(|&u| u.into()).collect(),
        };
        assert_eq!(mid_elem_sum(door.restored_updates()), 123);
    }
}
