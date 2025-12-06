use std::collections::HashMap;

use aoc_companion::prelude::*;
use aoc_utils::iter::AtMostTwo;
use itertools::Itertools;

pub(crate) struct Door {
    splitters: Vec<Vec<i32>>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        parse_splitters(input).map(|splitters| Door { splitters })
    }

    fn part1(&self) -> usize {
        split_beam(&self.splitters).splits
    }

    fn part2(&self) -> usize {
        split_beam(&self.splitters).quantum_worlds()
    }
}

fn parse_splitters(input: &str) -> Result<Vec<Vec<i32>>> {
    let mut lines = input.lines().step_by(2);
    let start_col = lines
        .next()
        .unwrap()
        .bytes()
        .position(|b| b == b'S')
        .unwrap() as i32;
    Ok(lines
        .map(|line| {
            line.bytes()
                .enumerate()
                .filter(|&(_, b)| b == b'^')
                .map(|(i, _)| i as i32 - start_col)
                .collect()
        })
        .collect())
}

#[derive(Debug, PartialEq, Eq)]
struct State {
    beams: HashMap<i32, usize>,
    splits: usize,
}

impl Default for State {
    fn default() -> Self {
        Self {
            beams: HashMap::from([(0, 1)]),
            splits: 0,
        }
    }
}

impl State {
    fn quantum_worlds(&self) -> usize {
        self.beams.values().sum()
    }
}

fn split_beam(splitters: &[impl AsRef<[i32]>]) -> State {
    splitters.iter().fold(
        State::default(),
        |State { beams, mut splits }, splitters| {
            let beams = beams
                .into_iter()
                .flat_map(|(b, c)| {
                    if splitters.as_ref().iter().contains(&b) {
                        splits += 1;
                        AtMostTwo::two((b - 1, c), (b + 1, c))
                    } else {
                        AtMostTwo::one((b, c))
                    }
                })
                .into_group_map()
                .into_iter()
                .map(|(b, cs)| (b, cs.into_iter().sum()))
                .collect();

            State { beams, splits }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "\
.......S.......
...............
.......^.......
...............
......^.^......
...............
.....^.^.^.....
...............
....^.^...^....
...............
...^.^...^.^...
...............
..^...^.....^..
...............
.^.^.^.^.^...^.
...............";

    const EXAMPLE_SPLITTERS: &[&[i32]] = &[
        &[0],
        &[-1, 1],
        &[-2, 0, 2],
        &[-3, -1, 3],
        &[-4, -2, 2, 4],
        &[-5, -1, 5],
        &[-6, -4, -2, 0, 2, 6],
    ];

    #[test]
    fn parse_example_splitters() {
        assert_eq!(parse_splitters(EXAMPLE_INPUT).unwrap(), EXAMPLE_SPLITTERS);
    }

    #[test]
    fn example_beam_split_count() {
        assert_eq!(split_beam(EXAMPLE_SPLITTERS).splits, 21);
    }

    #[test]
    fn example_quantum_worlds() {
        assert_eq!(split_beam(EXAMPLE_SPLITTERS).quantum_worlds(), 40);
    }
}
