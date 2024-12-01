use std::num::ParseIntError;

use aoc_companion::prelude::*;
use aoc_utils::iter::IterUtils;
use itertools::Itertools;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Door {
    left: Vec<usize>,
    right: Vec<usize>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ParseError {
    #[error("expected two whitespace-separated tokens, got {token_count} tokens")]
    WhitespaceError { token_count: usize },
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseError> {
        let (left, right) = input
            .lines()
            .map(|line| -> Result<(usize, usize), ParseError> {
                line.split_ascii_whitespace()
                    .collect_tuple()
                    .ok_or_else(|| ParseError::WhitespaceError {
                        token_count: line.split_ascii_whitespace().count(),
                    })
                    .and_then(|(lhs, rhs)| Ok((lhs.parse()?, rhs.parse()?)))
            })
            .try_unzip()?;

        Ok(Door { left, right })
    }

    fn part1(&self) -> usize {
        sorted_abs_diff(&self.left, &self.right)
    }

    fn part2(&self) -> usize {
        similarity_score(&self.left, &self.right)
    }
}

fn sorted_abs_diff(lhs: &[usize], rhs: &[usize]) -> usize {
    std::iter::zip(lhs.iter().sorted(), rhs.iter().sorted())
        .map(|(&lhs, &rhs)| lhs.abs_diff(rhs))
        .sum()
}

fn similarity_score(lhs: &[usize], rhs: &[usize]) -> usize {
    let rhs_counts = rhs.iter().copied().counts();
    lhs.iter()
        .map(|id| id * rhs_counts.get(id).unwrap_or(&0))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    const EXAMPLE_INPUT: &str = "3   4
4   3
2   5
1   3
3   9
3   3";
    const EXAMPLE_LIST_LEFT: &[usize] = &[3, 4, 2, 1, 3, 3];
    const EXAMPLE_LIST_RIGHT: &[usize] = &[4, 3, 5, 3, 9, 3];

    #[test]
    fn parse_fails_for_token_mismatch() {
        assert_matches!(
            Door::parse("1"),
            Err(ParseError::WhitespaceError { token_count: 1 })
        );
        assert_matches!(
            Door::parse("1 2 3"),
            Err(ParseError::WhitespaceError { token_count: 3 })
        );
        assert_matches!(
            Door::parse("one two three"),
            Err(ParseError::WhitespaceError { token_count: 3 })
        );
    }

    #[test]
    fn parse_fails_for_non_numbers() {
        assert_matches!(Door::parse("one two"), Err(ParseError::ParseIntError(_)));
        assert_matches!(Door::parse("one 2"), Err(ParseError::ParseIntError(_)));
        assert_matches!(Door::parse("1 two"), Err(ParseError::ParseIntError(_)));
    }

    #[test]
    fn parse_unzips_both_lists() {
        assert_eq!(
            Door::parse(EXAMPLE_INPUT).unwrap(),
            Door {
                left: EXAMPLE_LIST_LEFT.to_vec(),
                right: EXAMPLE_LIST_RIGHT.to_vec(),
            }
        );
    }

    #[test]
    fn abs_diff_of_sorted_lists() {
        assert_eq!(sorted_abs_diff(EXAMPLE_LIST_LEFT, EXAMPLE_LIST_RIGHT), 11);
    }

    #[test]
    fn example_similarity_scope() {
        assert_eq!(similarity_score(EXAMPLE_LIST_LEFT, EXAMPLE_LIST_RIGHT), 31);
    }
}
