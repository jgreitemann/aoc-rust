use anyhow::anyhow;
use aoc_companion::prelude::*;

use itertools::Itertools;

pub(crate) struct Door {
    passes: Vec<String>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        input
            .lines()
            .map(to_binary_string)
            .try_collect()
            .map(|passes| Door { passes })
    }

    fn part1(&self) -> u32 {
        self.passes
            .iter()
            .map(|bpass| seat_id_for_boarding_pass(bpass))
            .max()
            .unwrap()
    }

    fn part2(&self) -> Result<u32> {
        let seat_ids = self
            .passes
            .iter()
            .map(|bpass| seat_id_for_boarding_pass(bpass))
            .sorted();

        seat_ids
            .into_iter()
            .tuple_windows()
            .find(|(i, j)| j - i == 2)
            .map(|(i, _)| i + 1)
            .ok_or_else(|| anyhow!("no empty seat"))
    }
}

fn to_binary_string(s: &str) -> Result<String> {
    s.chars()
        .map(|c| match c {
            'F' | 'L' => Ok('0'),
            'B' | 'R' => Ok('1'),
            _ => Err(anyhow!("invalid 'digit': {c:?}")),
        })
        .try_collect()
}

fn seat_id_for_boarding_pass(bpass: &str) -> u32 {
    u32::from_str_radix(bpass, 2).unwrap()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn convert_boarding_passes_to_seat_id() {
        assert_eq!(
            seat_id_for_boarding_pass(&to_binary_string("FBFBBFFRLR").unwrap()),
            357
        );
        assert_eq!(
            seat_id_for_boarding_pass(&to_binary_string("BFFFBBFRRR").unwrap()),
            567
        );
        assert_eq!(
            seat_id_for_boarding_pass(&to_binary_string("FFFBBBFRRR").unwrap()),
            119
        );
        assert_eq!(
            seat_id_for_boarding_pass(&to_binary_string("BBFFBBFRLL").unwrap()),
            820
        );
    }
}
