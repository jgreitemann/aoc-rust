use std::{collections::VecDeque, num::ParseIntError};

use aoc_companion::prelude::*;

const FINAL_NUMBER: u32 = 2017;

pub(crate) struct Door {
    skip_len: usize,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseIntError> {
        Ok(Door {
            skip_len: input.parse()?,
        })
    }

    fn part1(&self) -> u32 {
        resulting_spinlock(FINAL_NUMBER, self.skip_len)[0]
    }

    fn part2(&self) -> u32 {
        find_value_after_zero(&resulting_spinlock(50_000_000, self.skip_len))
    }
}

fn skip_and_insert(ring: &mut VecDeque<u32>, next: u32, skip: usize) {
    if !ring.is_empty() {
        ring.rotate_left(skip % ring.len());
    }
    ring.push_back(next);
}

fn resulting_spinlock(final_number: u32, skip: usize) -> VecDeque<u32> {
    (0..=final_number).fold(VecDeque::new(), |mut ring, next| {
        skip_and_insert(&mut ring, next, skip);
        ring
    })
}

fn find_value_after_zero(ring: &VecDeque<u32>) -> u32 {
    let zero_pos = ring.iter().position(|&x| x == 0).unwrap();
    ring.get(zero_pos + 1).copied().unwrap_or_else(|| ring[0])
}

#[cfg(test)]
mod tests {
    use itertools::assert_equal;

    use super::*;

    const EXAMPLE_STEP: usize = 3;

    #[test]
    fn first_ten_example_steps() {
        assert_equal(
            (0..10).scan(VecDeque::new(), |ring, next| {
                skip_and_insert(ring, next, EXAMPLE_STEP);

                // rotate back such that 0 is in the first position
                let mut clone = ring.clone();
                let offset = ring.iter().position(|&x| x == 0).unwrap();
                clone.rotate_left(offset);
                Some(clone)
            }),
            [
                &[0],
                &[0, 1],
                &[0, 2, 1],
                &[0, 2, 3, 1],
                &[0, 2, 4, 3, 1],
                &[0, 5, 2, 4, 3, 1],
                &[0, 5, 2, 4, 3, 6, 1],
                &[0, 5, 7, 2, 4, 3, 6, 1],
                &[0, 5, 7, 2, 4, 3, 8, 6, 1],
                &[0, 9, 5, 7, 2, 4, 3, 8, 6, 1],
            ] as [&[u32]; 10],
        );
    }

    #[test]
    fn next_after_2017_in_example() {
        assert_eq!(resulting_spinlock(FINAL_NUMBER, EXAMPLE_STEP)[0], 638);
    }
}
