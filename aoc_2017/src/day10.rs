use aoc_companion::prelude::*;

use itertools::Itertools;
use tap::tap::Tap;
use thiserror::Error;

pub struct Door {
    lengths: Vec<usize>,
}

impl ParseInput<'_> for Door {
    type Error = std::num::ParseIntError;

    fn parse(input: &str) -> Result<Self, Self::Error> {
        let lengths = input.trim().split(',').map(str::parse).try_collect()?;
        Ok(Self { lengths })
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("The size of the string is insufficient to perform the operation")]
    NotEnoughElements,
    #[error("Result of multiplication overflowed")]
    Overflow,
}

impl Part1 for Door {
    type Output = usize;
    type Error = Error;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        product_of_first_two_elements(&apply_ties::<256>(&self.lengths))
    }
}

fn apply_ties<const N: usize>(lengths: &[usize]) -> KnotState<N> {
    lengths
        .into_iter()
        .fold(KnotState::new(), |state, &length| {
            state.tap_mut(|s| s.tie(length))
        })
}

fn product_of_first_two_elements<const N: usize>(state: &KnotState<N>) -> Result<usize, Error> {
    match state.marks[..] {
        [first, second, ..] => usize::checked_mul(first, second).ok_or(Error::Overflow),
        _ => Err(Error::NotEnoughElements),
    }
}

struct KnotState<const N: usize> {
    marks: [usize; N],
    skip_size: usize,
    current_pos: usize,
}

impl<const N: usize> KnotState<N> {
    fn new() -> Self {
        KnotState {
            marks: std::array::from_fn(|i| i),
            skip_size: 0,
            current_pos: 0,
        }
    }

    fn tie(&mut self, length: usize) {
        self.marks.rotate_left(self.current_pos);
        self.marks[..length].reverse();
        self.marks.rotate_right(self.current_pos);
        self.current_pos += length + self.skip_size;
        self.current_pos %= N;
        self.skip_size += 1;
    }
}

#[cfg(test)]
mod tests {
    use itertools::assert_equal;

    use super::*;

    const EXAMPLE_LENGTHS: &[usize] = &[3, 4, 1, 5];

    #[test]
    fn example_reproduces_intermediate_results() {
        assert_equal(
            EXAMPLE_LENGTHS
                .into_iter()
                .scan(KnotState::new(), |state, &length| {
                    state.tie(length);
                    Some(state.marks.clone())
                }),
            [
                [2, 1, 0, 3, 4],
                [4, 3, 0, 1, 2],
                [4, 3, 0, 1, 2],
                [3, 4, 2, 1, 0],
            ],
        );
    }

    #[test]
    fn example_product_of_first_two_elements() {
        assert_eq!(
            product_of_first_two_elements(&apply_ties::<5>(EXAMPLE_LENGTHS)).unwrap(),
            12
        );
    }
}
