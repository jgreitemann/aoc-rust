use std::collections::HashSet;

use aoc_companion::prelude::*;
use itertools::Itertools;

pub(crate) struct Door<'input> {
    groups: Vec<Vec<&'input str>>,
}

impl<'input> Solution<'input> for Door<'input> {
    fn parse(input: &'input str) -> Self {
        Door {
            groups: parse_groups(input),
        }
    }

    fn part1(&self) -> usize {
        self.groups.iter().map(|g| unique_answer_count(g)).sum()
    }

    fn part2(&self) -> usize {
        self.groups.iter().map(|g| common_answer_count(g)).sum()
    }
}

fn parse_groups(input: &str) -> Vec<Vec<&str>> {
    input
        .split("\n\n")
        .map(|group| group.lines().collect())
        .collect()
}

fn unique_answer_count(group: &[&str]) -> usize {
    group.iter().flat_map(|p| p.bytes()).unique().count()
}

fn common_answer_count(group: &[&str]) -> usize {
    group
        .iter()
        .map(|p| HashSet::<_, std::hash::RandomState>::from_iter(p.bytes()))
        .reduce(|lhs, rhs| lhs.intersection(&rhs).copied().collect())
        .map(|intersection| intersection.len())
        .unwrap_or(0)
}

#[cfg(test)]
mod test {
    use itertools::assert_equal;

    use super::*;

    const EXAMPLE_INPUT: &str = "\
abc

a
b
c

ab
ac

a
a
a
a

b";

    const EXAMPLE_GROUPS: &[&[&str]] = &[
        &["abc"],
        &["a", "b", "c"],
        &["ab", "ac"],
        &["a", "a", "a", "a"],
        &["b"],
    ];

    #[test]
    fn parse_example_input() {
        assert_eq!(parse_groups(EXAMPLE_INPUT), EXAMPLE_GROUPS);
    }

    #[test]
    fn unique_answers_for_example_groups() {
        assert_equal(
            EXAMPLE_GROUPS.iter().map(|g| unique_answer_count(g)),
            [3, 3, 3, 1, 1],
        );
    }

    #[test]
    fn common_answers_for_example_groups() {
        assert_equal(
            EXAMPLE_GROUPS.iter().map(|g| common_answer_count(g)),
            [3, 0, 1, 1, 1],
        );
    }
}
