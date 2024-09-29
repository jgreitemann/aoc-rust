use aoc_companion::prelude::*;

use itertools::Itertools;

use std::num::ParseIntError;

const DECRYPTION_KEY: isize = 811589153;

pub struct Door {
    numbers: Vec<isize>,
}

impl ParseInput<'_> for Door {
    type Error = ParseIntError;

    fn parse(input: &str) -> Result<Self, Self::Error> {
        parse_input(input).map(|numbers| Self { numbers })
    }
}

impl Part1 for Door {
    type Output = isize;
    type Error = std::convert::Infallible;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        Ok(sum_of_grove_coords(&decrypted_sequence(&self.numbers, 1)))
    }
}

impl Part2 for Door {
    type Output = isize;
    type Error = std::convert::Infallible;

    fn part2(&self) -> Result<Self::Output, Self::Error> {
        Ok(sum_of_grove_coords(&decrypted_sequence(
            &apply_key(&self.numbers),
            10,
        )))
    }
}

fn parse_input(input: &str) -> Result<Vec<isize>, ParseIntError> {
    input.lines().map(str::trim).map(str::parse).try_collect()
}

fn decrypted_sequence(numbers: &[isize], times: usize) -> Vec<isize> {
    use std::collections::VecDeque;

    let n = numbers.len() as isize - 1;
    let mut numbers = VecDeque::from_iter(numbers.iter().copied());
    let mut indices = Vec::from_iter(0..numbers.len());

    for i in (0..numbers.len()).cycle().take(numbers.len() * times) {
        let j = indices[i];
        let x = numbers.remove(j).unwrap();
        let to_move = (x % n + n) % n;
        let k = (j + to_move as usize) % numbers.len();

        numbers.insert(k, x);

        if j < k {
            indices
                .iter_mut()
                .filter(|l| (j + 1..=k).contains(l))
                .for_each(|l| *l -= 1);
        } else {
            indices
                .iter_mut()
                .filter(|l| (k..j).contains(l))
                .for_each(|l| *l += 1);
        }
        indices[i] = k;
    }

    numbers.into()
}

fn apply_key(numbers: &[isize]) -> Vec<isize> {
    numbers.iter().map(|x| x * DECRYPTION_KEY).collect()
}

fn sum_of_grove_coords(decrypted_seq: &[isize]) -> isize {
    decrypted_seq
        .iter()
        .cycle()
        .skip_while(|&&x| x != 0)
        .skip(1000)
        .step_by(1000)
        .take(3)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_is_parsed() {
        assert_eq!(parse_input(EXAMPLE_INPUT).unwrap(), EXAMPLE_SEQUENCE);
    }

    #[test]
    fn original_sequence_is_decrypted_once() {
        assert_eq!(
            decrypted_sequence(EXAMPLE_SEQUENCE, 1),
            EXAMPLE_DECRYPTED_SEQ
        );
    }

    #[test]
    fn sequence_with_key_applied_is_decrypted_ten_times() {
        assert_eq!(
            decrypted_sequence(&apply_key(EXAMPLE_SEQUENCE), 10),
            EXAMPLE_PART2_SEQ
        );
    }

    #[test]
    fn sum_of_grove_coords_is_found() {
        assert_eq!(sum_of_grove_coords(EXAMPLE_DECRYPTED_SEQ), 3);
    }

    const EXAMPLE_INPUT: &str = "\
        1
        2
        -3
        3
        -2
        0
        4";

    const EXAMPLE_SEQUENCE: &[isize] = &[1, 2, -3, 3, -2, 0, 4];
    const EXAMPLE_DECRYPTED_SEQ: &[isize] = &[-2, 1, 2, -3, 4, 0, 3];
    const EXAMPLE_PART2_SEQ: &[isize] = &[
        0,
        -2434767459,
        1623178306,
        3246356612,
        -1623178306,
        2434767459,
        811589153,
    ];
}
