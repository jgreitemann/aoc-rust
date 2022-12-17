use aoc_companion::prelude::*;
use aoc_utils::linalg::Vector;

use itertools::Itertools;
use thiserror::Error;

pub struct Door {
    jet_pattern: Vec<Jet>,
}

impl ParseInput<'_> for Door {
    type Error = ParseError;

    fn parse(input: &str) -> Result<Self, Self::Error> {
        parse_jet_pattern(input).map(|jet_pattern| Self {jet_pattern})
    }
}

impl Part1 for Door {
    type Output = usize;
    type Error = std::convert::Infallible;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        Ok(cavern_after_dropping_rocks(2022, &self.jet_pattern).height())
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Encountered an invalid character ({0:?}) in the jet pattern")]
    InvalidJetPatternChar(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Jet {
    Left,
    Right,
}

fn parse_jet_pattern(input: &str) -> Result<Vec<Jet>, ParseError> {
    input
        .chars()
        .map(|c| match c {
            '<' => Ok(Jet::Left),
            '>' => Ok(Jet::Right),
            _ => Err(ParseError::InvalidJetPatternChar(c)),
        })
        .try_collect()
}

struct Cavern {
    settled: ndarray::Array2<u8>,
}

impl Cavern {
    fn new() -> Self {
        Self {
            settled: ndarray::Array2::from_shape_vec(
                (8, 9),
                [
                    b"#########",
                    b"#.......#",
                    b"#.......#",
                    b"#.......#",
                    b"#.......#",
                    b"#.......#",
                    b"#.......#",
                    b"#.......#",
                ]
                .into_iter()
                .copied()
                .flatten()
                .collect(),
            )
            .unwrap(),
        }
    }

    fn height(&self) -> usize {
        self.settled.shape()[0] - 8
    }

    fn test(&self, rock: &Rock) -> bool {
        rock.0.iter().all(|&coords| self.settled[coords] == b'.')
    }

    fn add(&mut self, rock: &Rock) {
        for coords in &rock.0 {
            self.settled[*coords] = b'#';
        }

        let new_height = rock.0.iter().map(|coords| coords[0]).max().unwrap();
        while self.height() < new_height {
            self.settled.push_row(ndarray::ArrayView::from(b"#.......#")).unwrap();
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Rock(Vec<Vector<usize, 2>>);

impl Rock {
    fn translate(&mut self, dim: usize, by: isize) {
        for p in &mut self.0 {
            p[dim] = (p[dim] as isize + by) as usize;
        }
    }
}

fn drop_rock(cavern: &mut Cavern, mut rock: Rock, jet_iter: &mut dyn Iterator<Item=Jet>) {
    rock.translate(0, cavern.height() as isize);
    while cavern.test(&rock) {
        let jet = jet_iter.next().unwrap();
        match jet {
            Jet::Left => rock.translate(1, -1),
            Jet::Right => rock.translate(1, 1),
        }
        if !cavern.test(&rock) {
            match jet {
                Jet::Left => rock.translate(1, 1),
                Jet::Right => rock.translate(1, -1),
            }
        }
        rock.translate(0, -1);
    }

    rock.translate(0, 1);
    cavern.add(&rock);
}

fn rock_factory() -> impl Iterator<Item=Rock> {
    ROCK_SEQUENCE.iter().cycle().map(|rock| Rock(rock.to_vec()))
}

fn cavern_after_dropping_rocks(n: usize, jets: &[Jet]) -> Cavern {
    let mut cavern = Cavern::new();
    let mut jet_iter = jets.iter().copied().cycle();
    for rock in rock_factory().take(n) {
        drop_rock(&mut cavern, rock, &mut jet_iter);
    }
    cavern
}

const ROCK_SEQUENCE: [&[Vector<usize, 2>]; 5] = [
    &[
        Vector([4, 3]),
        Vector([4, 4]),
        Vector([4, 5]),
        Vector([4, 6]),
    ],
    &[
        Vector([5, 3]),
        Vector([5, 4]),
        Vector([4, 4]),
        Vector([6, 4]),
        Vector([5, 5]),
    ],
    &[
        Vector([4, 3]),
        Vector([4, 4]),
        Vector([4, 5]),
        Vector([5, 5]),
        Vector([6, 5]),
    ],
    &[
        Vector([4, 3]),
        Vector([5, 3]),
        Vector([6, 3]),
        Vector([7, 3]),
    ],
    &[
        Vector([4, 3]),
        Vector([4, 4]),
        Vector([5, 3]),
        Vector([5, 4]),
    ],
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jet_pattern_is_parsed() {
        assert_eq!(
            parse_jet_pattern(EXAMPLE_INPUT).unwrap().as_slice(),
            EXAMPLE_JET_PATTERN
        );
    }

    #[test]
    fn empty_cavern_has_height_zero() {
        assert_eq!(Cavern::new().height(), 0);
    }

    #[test]
    fn each_of_the_rocks_can_be_placed_in_an_empty_cavern() {
        let cavern = Cavern::new();
        assert!(ROCK_SEQUENCE
            .map(|rock| Rock(rock.to_vec()))
            .iter()
            .all(|rock| cavern.test(rock)));
    }

    #[test]
    fn after_adding_a_rock_to_the_cavern_the_height_increases() {
        let mut cavern = Cavern::new();
        cavern.add(&Rock(ROCK_SEQUENCE[2].to_vec()));
        assert_eq!(cavern.height(), 6);
    }

    #[test]
    fn after_dropping_the_first_rock_the_cavern_height_is_one() {
        assert_eq!(cavern_after_dropping_rocks(1, EXAMPLE_JET_PATTERN).height(), 1);
    }

    #[test]
    fn after_dropping_2022_rocks_the_example_cavern_height_is_reached() {
        assert_eq!(cavern_after_dropping_rocks(2022, EXAMPLE_JET_PATTERN).height(), 3068);
    }

    const EXAMPLE_INPUT: &str = ">>><<><>><<<>><>>><<<>>><<<><<<>><>><<>>";

    const EXAMPLE_JET_PATTERN: &[Jet] = &[
        Jet::Right,
        Jet::Right,
        Jet::Right,
        Jet::Left,
        Jet::Left,
        Jet::Right,
        Jet::Left,
        Jet::Right,
        Jet::Right,
        Jet::Left,
        Jet::Left,
        Jet::Left,
        Jet::Right,
        Jet::Right,
        Jet::Left,
        Jet::Right,
        Jet::Right,
        Jet::Right,
        Jet::Left,
        Jet::Left,
        Jet::Left,
        Jet::Right,
        Jet::Right,
        Jet::Right,
        Jet::Left,
        Jet::Left,
        Jet::Left,
        Jet::Right,
        Jet::Left,
        Jet::Left,
        Jet::Left,
        Jet::Right,
        Jet::Right,
        Jet::Left,
        Jet::Right,
        Jet::Right,
        Jet::Left,
        Jet::Left,
        Jet::Right,
        Jet::Right,
    ];
}
