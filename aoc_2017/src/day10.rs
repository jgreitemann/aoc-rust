use aoc_companion::prelude::*;

use itertools::Itertools;
use tap::tap::Tap;
use thiserror::Error;

use std::num::ParseIntError;

pub struct Door<'input> {
    input: &'input str,
}

impl<'input> ParseInput<'input> for Door<'input> {
    type Error = std::convert::Infallible;

    fn parse(input: &'input str) -> Result<Self, Self::Error> {
        Ok(Self { input })
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("The size of the string is insufficient to perform the operation")]
    NotEnoughElements,
    #[error(transparent)]
    ParseError(#[from] ParseIntError),
}

impl Part1 for Door<'_> {
    type Output = u16;
    type Error = Error;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        let lengths = as_list_of_numbers(&self.input)?;
        product_of_first_two_elements(&apply_ties::<256>(lengths.into_iter()))
    }
}

fn as_list_of_numbers(input: &str) -> Result<Vec<u8>, ParseIntError> {
    input.trim().split(',').map(str::parse).try_collect()
}

fn apply_ties<const N: usize>(lengths: impl Iterator<Item = u8>) -> KnotState<N> {
    lengths.fold(KnotState::new(), |state, length| {
        state.tap_mut(|s| s.tie(length as usize))
    })
}

fn product_of_first_two_elements<const N: usize>(state: &KnotState<N>) -> Result<u16, Error> {
    match state.marks[..] {
        [first, second, ..] => Ok(u16::overflowing_mul(first as u16, second as u16).0),
        _ => Err(Error::NotEnoughElements),
    }
}

struct KnotState<const N: usize> {
    marks: [u8; N],
    skip_size: usize,
    current_pos: usize,
}

impl<const N: usize> KnotState<N> {
    fn new() -> Self {
        KnotState {
            marks: std::array::from_fn(|i| i as u8),
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

    const EXAMPLE_LENGTHS: &[u8] = &[3, 4, 1, 5];

    #[test]
    fn example_reproduces_intermediate_results() {
        assert_equal(
            EXAMPLE_LENGTHS
                .into_iter()
                .scan(KnotState::new(), |state, &length| {
                    state.tie(length as usize);
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
            product_of_first_two_elements(&apply_ties::<5>(EXAMPLE_LENGTHS.iter().copied()))
                .unwrap(),
            12
        );
    }
}
