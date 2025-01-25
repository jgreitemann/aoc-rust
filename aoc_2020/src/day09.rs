use std::num::ParseIntError;

use anyhow::anyhow;
use aoc_companion::prelude::*;

use itertools::MinMaxResult::MinMax;
use itertools::{Itertools, MinMaxResult};

pub(crate) struct Door(Vec<i64>);

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseIntError> {
        input.lines().map(str::parse).try_collect().map(Door)
    }

    fn part1(&self) -> Result<i64> {
        find_first_offending_number(&self.0, 25)
    }

    fn part2(&self) -> Result<i64> {
        let offender = find_first_offending_number(&self.0, 25)?;
        let slice = find_slice_with_sum(&self.0, offender)?;
        Ok(sum_of_min_and_max(slice).unwrap())
    }
}

fn is_sum_of_two_numbers(x: i64, nums: &[i64]) -> bool {
    // x=6
    // nums=[2,3,7,3,4,-1]
    nums.iter().any(|&lhs| {
        let rhs = x - lhs;
        lhs != rhs && nums.contains(&rhs)
    })
}

fn find_first_offending_number(nums: &[i64], n: usize) -> Result<i64> {
    nums.windows(n + 1)
        .find_map(|window| {
            let (&elem, predecessors) = window.split_last().unwrap();
            (!is_sum_of_two_numbers(elem, predecessors)).then_some(elem)
        })
        .ok_or_else(|| {
            anyhow!("did not find a number that isn't the sum of two of the preceding {n} numbers")
        })
}

fn find_slice_with_sum(source: &[i64], sum: i64) -> Result<&[i64]> {
    (2..source.len())
        .flat_map(|window_size| source.windows(window_size))
        .find(|window| window.iter().sum::<i64>() == sum)
        .ok_or_else(|| anyhow!("did not find a contiguous range"))
}

fn sum_of_min_and_max(nums: &[i64]) -> Option<i64> {
    match nums.iter().minmax() {
        MinMaxResult::NoElements => None,
        MinMaxResult::OneElement(&single) => Some(single),
        MinMax(min, max) => Some(min + max),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sum_of_two_numbers() {
        assert!(!is_sum_of_two_numbers(2, &[]));
        assert!(!is_sum_of_two_numbers(8, &[2, 3, 7]));
        assert!(is_sum_of_two_numbers(5, &[2, 3, 7]));
        assert!(!is_sum_of_two_numbers(6, &[2, 3, 7]));
        assert!(is_sum_of_two_numbers(6, &[2, 3, 7, 4]));

        let first_25_nums = (1..=25).collect_vec();
        assert!(is_sum_of_two_numbers(26, first_25_nums.as_slice()));
        assert!(is_sum_of_two_numbers(49, first_25_nums.as_slice()));
        assert!(!is_sum_of_two_numbers(100, first_25_nums.as_slice()));
        assert!(!is_sum_of_two_numbers(50, first_25_nums.as_slice()));
    }

    const EXAMPLE: [i64; 20] = [
        35, 20, 15, 25, 47, 40, 62, 55, 65, 95, 102, 117, 150, 182, 127, 219, 299, 277, 309, 576,
    ];

    #[test]
    fn first_offending_number_found() {
        assert_eq!(find_first_offending_number(&EXAMPLE, 5).unwrap(), 127);

        find_first_offending_number(&(1..=5).collect_vec(), 3).unwrap_err();

        assert_eq!(
            find_first_offending_number(&(1..=10).collect_vec(), 3).unwrap(),
            6
        );
    }

    #[test]
    fn slice_with_sum_found() {
        const SUBSET: [i64; 4] = [15, 25, 47, 40];
        assert_eq!(find_slice_with_sum(&EXAMPLE, 127).unwrap(), SUBSET);
    }
}
