use std::num::ParseIntError;

use aoc_companion::prelude::*;
use itertools::Itertools;

pub(crate) struct Door {
    reports: Vec<Vec<i32>>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseIntError> {
        Ok(Self {
            reports: parse_reports(input)?,
        })
    }

    fn part1(&self) -> usize {
        self.reports
            .iter()
            .map(assess_report)
            .filter(Result::is_ok)
            .count()
    }

    fn part2(&self) -> usize {
        self.reports
            .iter()
            .map(|r| assess_report_with_safety_dampener(r))
            .filter(Option::is_some)
            .count()
    }
}

fn parse_reports(input: &str) -> Result<Vec<Vec<i32>>, ParseIntError> {
    input
        .lines()
        .map(|line| line.split_ascii_whitespace().map(str::parse).try_collect())
        .try_collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SafeChange {
    Empty,
    SafelyIncreasing,
    SafelyDecreasing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnsafeChange {
    None,
    TooBig,
    Erratic,
}

fn assess_report<'l>(
    report: impl IntoIterator<Item = &'l i32>,
) -> Result<SafeChange, UnsafeChange> {
    report
        .into_iter()
        .tuple_windows()
        .map(|(lhs, rhs)| rhs - lhs)
        .try_fold(SafeChange::Empty, |prev, diff| {
            let this_change = match diff {
                -3..=-1 => Ok(SafeChange::SafelyDecreasing),
                1..=3 => Ok(SafeChange::SafelyIncreasing),
                0 => Err(UnsafeChange::None),
                _ => Err(UnsafeChange::TooBig),
            };
            this_change.and_then(|trend| {
                if prev == SafeChange::Empty || prev == trend {
                    Ok(trend)
                } else {
                    Err(UnsafeChange::Erratic)
                }
            })
        })
}

fn without_level_iter(report: &[i32], omitted_index: usize) -> impl Iterator<Item = &i32> {
    report[..omitted_index]
        .iter()
        .chain(report[omitted_index..].iter().skip(1))
}

fn assess_report_with_safety_dampener(report: &[i32]) -> Option<SafeChange> {
    std::iter::once(assess_report(report))
        .chain(
            (0..report.len())
                .map(|omitted_index| assess_report(without_level_iter(report, omitted_index))),
        )
        .find_map(|assessment| assessment.ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "7 6 4 2 1
1 2 7 8 9
9 7 6 2 1
1 3 2 4 5
8 6 4 4 1
1 3 6 7 9";

    const EXAMPLE_REPORTS: &[&[i32]] = &[
        &[7, 6, 4, 2, 1],
        &[1, 2, 7, 8, 9],
        &[9, 7, 6, 2, 1],
        &[1, 3, 2, 4, 5],
        &[8, 6, 4, 4, 1],
        &[1, 3, 6, 7, 9],
    ];

    #[test]
    fn example_reports_input_parsed() {
        assert_eq!(parse_reports(EXAMPLE_INPUT).unwrap(), EXAMPLE_REPORTS);
    }

    #[test]
    fn example_report_assessments_without_safety_dampener() {
        assert_eq!(
            assess_report(EXAMPLE_REPORTS[0]),
            Ok(SafeChange::SafelyDecreasing)
        );
        assert_eq!(assess_report(EXAMPLE_REPORTS[1]), Err(UnsafeChange::TooBig));
        assert_eq!(assess_report(EXAMPLE_REPORTS[2]), Err(UnsafeChange::TooBig));
        assert_eq!(
            assess_report(EXAMPLE_REPORTS[3]),
            Err(UnsafeChange::Erratic)
        );
        assert_eq!(assess_report(EXAMPLE_REPORTS[4]), Err(UnsafeChange::None));
        assert_eq!(
            assess_report(EXAMPLE_REPORTS[5]),
            Ok(SafeChange::SafelyIncreasing)
        );
    }

    #[test]
    fn example_report_assessments_with_safety_dampener() {
        assert_eq!(
            assess_report_with_safety_dampener(EXAMPLE_REPORTS[0]),
            Some(SafeChange::SafelyDecreasing)
        );
        assert_eq!(assess_report_with_safety_dampener(EXAMPLE_REPORTS[1]), None);
        assert_eq!(assess_report_with_safety_dampener(EXAMPLE_REPORTS[2]), None);
        assert_eq!(
            assess_report_with_safety_dampener(EXAMPLE_REPORTS[3]),
            Some(SafeChange::SafelyIncreasing)
        );
        assert_eq!(
            assess_report_with_safety_dampener(EXAMPLE_REPORTS[4]),
            Some(SafeChange::SafelyDecreasing)
        );
        assert_eq!(
            assess_report_with_safety_dampener(EXAMPLE_REPORTS[5]),
            Some(SafeChange::SafelyIncreasing)
        );
    }
}
