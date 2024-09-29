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
        let lengths = as_list_of_numbers(self.input)?;
        product_of_first_two_elements(&apply_ties::<256>(lengths.into_iter()))
    }
}

impl Part2 for Door<'_> {
    type Output = String;
    type Error = std::convert::Infallible;

    fn part2(&self) -> Result<Self::Output, Self::Error> {
        Ok(knot_hash(self.input))
    }
}

fn as_list_of_numbers(input: &str) -> Result<Vec<u8>, ParseIntError> {
    input.split(',').map(str::parse).try_collect()
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

fn sparse_hash<const N: usize>(input: &str) -> [u8; N] {
    let lengths = itertools::repeat_n(
        input
            .as_bytes()
            .iter()
            .copied()
            .chain([17u8, 31u8, 73u8, 47u8, 23u8]),
        64,
    )
    .flatten();

    apply_ties(lengths).marks
}

fn densify(sparse_hash: &[u8]) -> impl Iterator<Item = u8> + '_ {
    sparse_hash.chunks(16).map(|chunk| {
        chunk
        .iter()
        .copied()
        .reduce(std::ops::BitXor::bitxor)
        .unwrap()
    })
}

fn knot_hash(input: &str) -> String {
    format!("{:02x}", densify(&sparse_hash::<256>(input)).format(""))
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
                .iter()
                .scan(KnotState::new(), |state, &length| {
                    state.tie(length as usize);
                    Some(state.marks)
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

    #[test]
    fn sparse_hash_is_densified() {
        assert_equal(
            densify(&[65, 27, 9, 1, 4, 3, 40, 50, 91, 7, 6, 0, 2, 5, 68, 22]),
            [64],
        );
    }

    #[test]
    fn example_knot_hashes() {
        assert_eq!(knot_hash(""), "a2582a3a0e66e6e86e3812dcb672a272");
        assert_eq!(knot_hash("AoC 2017"), "33efeb34ea91902bb2f59c9920caa6cd");
        assert_eq!(knot_hash("1,2,3"), "3efbe78a8d82f29979031a4aa0b16a9d");
        assert_eq!(knot_hash("1,2,4"), "63960835bcdc130f0b66d7ff4f6a5a8e");
    }
}
