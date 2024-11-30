use aoc_companion::prelude::*;
use aoc_utils::linalg::Vector;

use itertools::Itertools;
use thiserror::Error;

use std::iter::Peekable;

pub(crate) struct Door {
    jet_pattern: Vec<Jet>,
}

impl<'input> ParseInput<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseError> {
        parse_jet_pattern(input).map(|jet_pattern| Self { jet_pattern })
    }
}

impl Part1 for Door {
    fn part1(&self) -> isize {
        cavern_after_dropping_rocks(2022, &self.jet_pattern).height()
    }
}

impl Part2 for Door {
    fn part2(&self) -> isize {
        determine_tower_height_with_matching(1000000000000, &self.jet_pattern)
    }
}

#[derive(Debug, Error)]
pub(crate) enum ParseError {
    #[error("Encountered an invalid character ({0:?}) in the jet pattern")]
    InvalidJetPatternChar(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Jet {
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

#[derive(Debug, Clone)]
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

    fn height(&self) -> isize {
        self.settled.shape()[0] as isize - 8
    }

    fn test(&self, rock: &Rock) -> bool {
        rock.0
            .iter()
            .all(|coords| self.settled[coords.try_cast_as().unwrap()] == b'.')
    }

    fn add(&mut self, rock: &Rock) {
        for coords in &rock.0 {
            self.settled[coords.try_cast_as().unwrap()] = b'#';
        }

        let new_height = rock.0.iter().map(|coords| coords[0]).max().unwrap();
        while self.height() < new_height {
            self.settled
                .push_row(ndarray::ArrayView::from(b"#.......#"))
                .unwrap();
        }
    }

    fn matches(&self, reference: &Cavern, check_amount: isize) -> bool {
        use ndarray::s;
        let reference_top = reference.settled.slice(s![-check_amount.., ..]);
        let self_top = self.settled.slice(s![-check_amount.., ..]);
        self_top == reference_top
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Rock(Vec<Vector<isize, 2>>);

impl Rock {
    fn translate(&mut self, by: Vector<isize, 2>) {
        for p in &mut self.0 {
            *p += by;
        }
    }
}

fn drop_rock(
    cavern: &mut Cavern,
    mut rock: Rock,
    jet_iter: &mut dyn Iterator<Item = (usize, Jet)>,
) {
    rock.translate(Vector([cavern.height(), 0]));
    while cavern.test(&rock) {
        let (_, jet) = jet_iter.next().unwrap();
        match jet {
            Jet::Left => rock.translate(Vector([0, -1])),
            Jet::Right => rock.translate(Vector([0, 1])),
        }
        if !cavern.test(&rock) {
            match jet {
                Jet::Left => rock.translate(Vector([0, 1])),
                Jet::Right => rock.translate(Vector([0, -1])),
            }
        }
        rock.translate(Vector([-1, 0]));
    }

    rock.translate(Vector([1, 0]));
    cavern.add(&rock);
}

fn drop_rocks_commensurate_to_jets<R, J>(
    cavern: &mut Cavern,
    rocks: R,
    jet_iter: &mut Peekable<J>,
) -> usize
where
    R: Iterator<Item = Rock> + Clone,
    J: Iterator<Item = (usize, Jet)>,
{
    let mut n = 0;
    let first_jet_idx = jet_iter.peek().unwrap().0;
    loop {
        for rock in rocks.clone() {
            drop_rock(cavern, rock, jet_iter);
            n += 1;
        }
        if jet_iter.peek().unwrap().0 == first_jet_idx {
            break;
        }
    }
    n
}

fn rock_factory() -> impl Iterator<Item = Rock> + Clone {
    ROCK_SEQUENCE.iter().map(|rock| Rock(rock.to_vec()))
}

fn drop_multiple_rocks(
    cavern: &mut Cavern,
    n: usize,
    jet_iter: &mut dyn Iterator<Item = (usize, Jet)>,
) {
    for rock in rock_factory().cycle().take(n) {
        drop_rock(cavern, rock, jet_iter);
    }
}

fn cavern_after_dropping_rocks(n: usize, jets: &[Jet]) -> Cavern {
    let mut cavern = Cavern::new();
    drop_multiple_rocks(
        &mut cavern,
        n,
        &mut jets.iter().copied().enumerate().cycle(),
    );
    cavern
}

fn determine_tower_height_with_matching(n: usize, jets: &[Jet]) -> isize {
    const SEED_N: usize = 25;

    let mut jet_iter = jets.iter().copied().enumerate().cycle().peekable();
    let mut cavern = Cavern::new();

    // Initially, fill the cavern with some amount of rocks to rule out transient effects from the straight floor.
    let mut initial_rocks = SEED_N * ROCK_SEQUENCE.len();
    drop_multiple_rocks(&mut cavern, initial_rocks, &mut jet_iter);
    // Then drop a number of rocks commensurate to the jet pattern to ensure that we have sufficient reference data.
    initial_rocks += drop_rocks_commensurate_to_jets(&mut cavern, rock_factory(), &mut jet_iter);
    let initial_cavern = cavern.clone();

    // Again drop a commensurate amount to determine how many rows to look at when matching.
    let mut rocks_until_repeat =
        drop_rocks_commensurate_to_jets(&mut cavern, rock_factory(), &mut jet_iter);
    let number_of_rows_to_match = cavern.height() - initial_cavern.height();

    // Repeat this until a match has been found. The segment of the tower by which it grew
    // since `initial_cavern` is bound to repeat over and over.
    while !cavern.matches(&initial_cavern, number_of_rows_to_match) {
        rocks_until_repeat +=
            drop_rocks_commensurate_to_jets(&mut cavern, rock_factory(), &mut jet_iter);
    }
    let matching_cavern = cavern.clone();
    let repeating_segment_height = matching_cavern.height() - initial_cavern.height();

    // We pretend as though we repeated this segment until just shy of the target amount of
    // rocks had been placed. The remaining blocks will correspond to a partial segment and
    // we determine the addition height through those by placing them on our tower (which in
    // reality contains the repeating segment just twice).
    let number_of_repeats = (n - initial_rocks) / rocks_until_repeat;
    let remaining_rocks = (n - initial_rocks) % rocks_until_repeat;
    drop_multiple_rocks(&mut cavern, remaining_rocks, &mut jet_iter);
    let remaining_height = cavern.height() - matching_cavern.height();

    initial_cavern.height()
        + number_of_repeats as isize * repeating_segment_height
        + remaining_height
}

const ROCK_SEQUENCE: [&[Vector<isize, 2>]; 5] = [
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
        assert_eq!(
            cavern_after_dropping_rocks(1, EXAMPLE_JET_PATTERN).height(),
            1
        );
    }

    #[test]
    fn after_dropping_2022_rocks_the_example_cavern_height_is_reached() {
        assert_eq!(
            cavern_after_dropping_rocks(2022, EXAMPLE_JET_PATTERN).height(),
            3068
        );
    }

    #[test]
    fn example_cavern_height_is_found_through_matching() {
        assert_eq!(
            determine_tower_height_with_matching(2022, EXAMPLE_JET_PATTERN),
            3068
        );
        assert_eq!(
            determine_tower_height_with_matching(1000000000000, EXAMPLE_JET_PATTERN),
            1514285714288
        );
        assert_eq!(
            determine_tower_height_with_matching(
                1000000000000,
                parse_jet_pattern(REAL_INPUT).unwrap().as_slice()
            ),
            1542941176480
        );
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

    const REAL_INPUT: &str = "\
    >>><>><<<<><<<>>><><<<<>>>><<<><>><>>><<<><<>>><<<><<><>>><<<><<<<><>>>><<<<>><<>>>><<<>><<<<>>><<<>>>><<<<>>>><<<\
    ><>>>><>>><><<>>><<<<>><<>>>><<><<>><<<<><<<><<<>><<>>><<<>>>><<<<><<<>><<><<<>><>>><<>>><<<<>>><><<<<>><<<<>><>>>\
    ><><>>><<<<><<>>><<<<>><<<<>>>><<<>><<>><<<<>>>><<><<<<>>>><<<<>>><<<<>>><<<>>>><><>><>>>><<<>>>><<>><<>>>><<<<>><\
    >>><><<<<>>>><<<<><<><><<<>>>><<<><<>>><<<<>>><<>>><>>>><><<<>>><><<<>>>><>>><<><>><<<>><<<<>><<<<>><>><<<<><>>>><\
    >>><<<>><>><<<<>>><<>><<<><<<<>>><>>><>>><>>><<>>><<<<>><<<>><<<<>>><<>><<<>><>>><<<<>>>><<<<><<>>><<<>>>><<<>>><<\
    <>><<<<>>><<><>><<>><<>>><<<>>><<<>>>><<<<>>>><<<<>>><<<<>><<<>>>><><<>>><>>>><<<<>>>><<<>>>><<<<><><>>><<><<<>>>>\
    <<<><<<<><<>><<<><<>>><<<<><<><<>><<<>>><<>>><<<>>>><<><<<<><<<>><>>>><<<<>>><<<><<<<>><<<>><<<<>><>><<<>>>><<<>><\
    <>>>><<<<>>>><<>>><>>><>>>><<>>><<<>>><<<>>><<<><><<>>>><><<<>>>><<<<>>>><>><<>><<<>>><>>>><<>>>><><>><>>><<<><<<>\
    >><<<<><<>><<<<>>><<<>><>>>><<<<>><<<<>>><<>>>><<<<>>><<<>>><<<><<<><<>><><<><<<<>>><<<>>>><<<<><<<>>><><<<<>>>><<\
    <>>>><<<><<<>>>><<<<>>>><<<>>>><<<>>><>><<>>>><<<<>><<>><<<<>>>><<><<<<>><<<<>><<<>>>><<<><<<>>><<>>><<<<>><>>><>>\
    >><>><>>>><<><<>><<<<>>><<>><>><>><<<>><<>>>><<>>><>>>><>>><>>>><>>><<<>>><<<<><><<<>>><<<>>><<<>>>><<><<>>><>>><>\
    >><>>><<<<>><<<<>>><<>>><<>><>>><<>>>><<>>><<>>><<>><>>>><<><<>>><<<>><<<<>>>><<><<>>>><<<>><<<><<<<>>>><<<<><><<>\
    >><>>><<<>><>><>><<<<>>><>>><>>>><<<<>>>><><<<><<<><>>>><<<>>><>><<<<>><<<>>><<<<>><<<><<<>>><<>><>>>><>>><<<<>>>>\
    <><<><<>>><<<<>><<<<><>>>><<<>>><<<<>><<<><><<<<>><<<<>>><>>>><>>>><<>>><<<<>><<<>><<<><<>><<<>>><<><<<<><<<<>>><<\
    <<>>>><<>>><>>><<><<>>>><<>>><>>>><<<><<>>>><<<>>><<>>><<>><<<<>>><><>>>><<<>>>><<<<>>><<<>>><>><>>>><>><>><<<<>><\
    <<>>><<<<>>>><<>><><<<>>><<<<>><>>><>>><><<>><<<>>><>>>><>>>><<><<<>><>><<<<>>>><<<<>><<>>><<>>>><>><<<<>><><<>>><\
    <<>><<<<><<>>>><<<><>>>><<<>><<<><<<>>><<>><<>><<<>>>><<<<>>><<<<><<<>><<<<><<>><<<<>><<<><>><<>>>><<>><<<<>><<>>>\
    <>><<>><<>>>><<><<<>><<<<><<<<>>><<<>>>><>>><<<>>><><<<>><<<<>>><<<<><<<>>>><<><<<><<>>>><>>><<>><<<<>>><>><<<<>><\
    ><>>>><>><<>><<<>>><<<>>>><><<>>>><<>>>><<<<>><>><<<>><<<><<<<><<>>><><<<<>>><>>>><<<>>><<<<>>><>>>><><<<<>>><><<<\
    <>><<<><<<>>><<<>><<>>><>><<>>>><>><<>><<<<>><<>>>><<>>><<<>><>>><<><<<<>><<<>><>>><<<<><>><<<<>><><<<<>><<<>>>><<\
    <<>><<<<>>><<<<><<>><<><<<<><><<>>><>>>><<<<>>><<>>>><<>><><>>><<<<>>><><>>><<<>><<>><<>><<<<>>>><<<>>><<<>><>>><<\
    <<>><<><>>>><<<>>>><<<>>><>>>><<<<><>>>><>><<<<>><<<>><<>>><<><<>>><<>>><<<>>><>>><<<>>><<><<><>>>><<<>><<<<>>><>>\
    <<>><<<>>>><>>><<<<>><>>>><<<<>><<<>><<<>>>><><<>>><<>><<>><<>>><>><<<><<<<>><>>><<>><<<>><>>>><<<<>><<<>>><<<<>>>\
    ><<<>><<<<>>>><>><>><<<<><<<<>>>><<>><>><<<<>>><<>>><<<>>><<>>>><<<>>>><>>><>><<<>>><><<<>><>>><<<><>><<<>><<<><<<\
    <>>><<><<<>>>><<<<><<>>><<<<>><><<<><<>><<<<>>>><<<<>><<<<>><<>>>><<>><<<<><<<>>><<>>><<<<>>>><<><><<<>>><<>><>>><\
    <<<>>><<<>>><><<<<><<>><<<><<>><<<<>>><<>>><<<<>>>><<<<><<>><<>>><>>><<<<><<<<>><<>>>><><><<>>>><>>><>><<<<>>>><<<\
    <>><<<<>>>><<<<>>>><<<<><<<>>>><>><<>>>><>>>><>>><<<><><>>>><<>>>><<<>>>><<>>>><<<>><<<>>>><<<<>><<>>><>><<<<>>>><\
    <<><<>>><<><<<<>>><<>>><<<>>><<<<>>>><<<<>>>><<>>>><<<>>>><>><<<<>>><<<<><<<<>>><<<<>><<<<><><<<<>>><<>>><<<>><<<>\
    >>><<<<>><<>>><<<>>><<<<>>>><>><<<<><<<<>>>><<<<>><<<>>><<<<>>>><<>>><>><><<>>><<<>>>><<<<>><<><<<>>>><>>>><<><<<>\
    ><<<>><<<>>><<<<>>>><>><<><<<<>>>><<<<>>>><>><<<<>><<>><<>><<<><<<<>><<><<<><<>><><<<>>><<<<><>><>><><<<>>><<>>><>\
    <<<>><>>>><>>>><>><>><>><>>>><<<<>>><>>>><>>>><<>>>><<<<>>><<><<<>>>><<<>>>><<<<>><<<<>><<><<>><>>>><<<>>>><<<>>><\
    >><<>>>><<>><<>>><>>>><<>>><>>><<<>><>><<<>>>><<<<>>>><<<>><<<<>>><<<<>>><>>>><<>>><<>>><<>>><>>>><<<<>>><<>><>>>>\
    <>>><<<<>>><<>>><>>><<<<>><<><>>><<><<<<>>>><<<<><<>>>><<<<>>><>><<<>>>><<>><<<<><<<<>>><>>>><<<<>>>><<<<>><>><>>>\
    ><<<<><<<<>>><<>><<<<>>>><<<><>>>><<<<><<>>>><<<<>>><<<<><><<<<>>><><<<>><>>><<<<>><<><<<<>><>>><<<<>>>><<<<>><<><\
    <>>><<<<>>><<>><<<<>><>>><<>>>><>><>><<<<>><<<>><<<>><<>>><>><<>>><>>>><<>><>><<<<>>><<<><<<><><<<<>>><<<>><<<<><<\
    >><<<>><<<<><>>>><<><<<>><><<<>>><<<>>><<<<>>><<<<><><<><>>><><>><><>>>><<>>>><<><>>><<>>>><><>>>><<><<<<>>>><>>><\
    <>>><<<><<<>>><<<<>>>><<<<><<>>>><>>><>>>><<<<><<<>><<<<>>>><<<>>>><<<<>>><<<<><<<><<<<><<<<>>><<<<>>><<<>>>><>>><\
    <<>>>><<<>><<<<>>>><<<<>>><<<<><<<<>>><<>>>><<<><<<<><<<>>>><<<><<<<><<<<><>>>><><<<<>>>><<>>>><>><>>><<>>>><<<<><\
    <<><>><<>>>><<<<>>><<><>>>><<<<>>><>>><<<<>>><>>><<<>>>><>>><<<<><<<<>><>>><>>>><>>>><<<><><>><<>>>><<<<><><<<<>>>\
    <<<><>><<>>>><<<<>>>><<<<>>>><<<<>><<>>><<<<>>>><<<<><<<>><>>>><>><<<><<<>>><>><<<<><<<<>><<>><<>><<>>>><>><<<<>>>\
    <<<<>>>><<<<><<><>>><<<<>>>><>><<<>>><><<>>><<<<>>>><>><<>><<<>><>><><<>>><<<<>>><<><<<<><<>>>><<<<>>>><<<>>>><><>\
    >>><<><<<><<>>><<>>>><<<>>>><<<>><<<<>><>>><><<<<>>>><>>><<<<>>>><>><<>>><>>>><<<>>>><<><<>>><<<>>><<>><>><<>>>><<\
    <<>><>><<<<>><<><<><<<<>>>><<<<><<<>><>>><<<<><>><<<><<>><><><<>>>><<<<>>><<<><<<<><<>>><<<>>><<>>>><>>>><<>>><<>>\
    >><<<<>><<>>><>>><><<>><<<<>>><<<><<<>><<>><<<>>><>>><<<<>><<<<><>>><>><<>>><<<>>>><<<<>>><>>><>>><<>>>><<<>><<<>>\
    ><>><<>>>><<>><<<<>><>><>>><><<>>><<<>><<<>>><<>>><<<<>>>><<><<<<>>><>><<<<>>>><<<<><>>><<<<>>><<><<<<>><<<<>><<>>\
    >><>>><<><>>>><<<<>>><<<><<<>>>><<>>>><<<<>>><>><<<<><<<<>>>><>><<<>><>>>><>>>><><<<>>><<>>><<>><<<<>>><<<><<<>><<\
    <>><<<>><<<<>>>><>><<>>>><>><<<<>>><><<<<><<><>><<<<>>>><<<><>>>><<<<>>><<>>>><<<>>>><<<>><<>><<<><<<<>><<<>>>><<>\
    ><><<<>>>><<<>><<<<>><>>>><<<>><<>><<>><<<>>><<>>>><<>>><>><>>>><<<<>>><>>><<>>>><><<<>><<<<>>>><>>><><<<>><<<>>><\
    <<<>>><>>>><><<>>>><<<><<<<>>><<<>><>>><><<>>><>>>><<><><<><<<<>>>><>><>><<>>><><<<<>>>><<<<>>><>>>><<>>><<<>><>><\
    <>>>><<><>>>><<<<>>><<<>>><<<<><<<<><>><<<>><<<>>>><<<<>><<<<>>><>>><<><<<<>>><<<><<><<<>>><<<<>>>><<><<<<>>><>><<\
    >>>><<><<<>>>><<><<<>><<<<><<<<>>><<<<>>>><<<<>><>>>><<<><<<>>>><<<>>>><<<>>><>><>><<<>><<><>>>><<<<>><<<>>><<<><>\
    <<>>>><>><>>><>>>><<<>>>><<>>>><<>>>><<><>>><<>>><<>>>><<>>><><<<<>>>><<<>>><<<<>><<>>><<<<>><>>><>>>><>><><<><<<<\
    >>>><>><<<<>>><<><<>>>><<>>><<<<>>><<<<>>>><<<>>>><<>>><<>>><<<><>>><<<><<>><<>><<<<><<<><<<<><<<><<<<>>>><<<>><>>\
    <<>>>><<<<>>>><<>>>><>>><<>>><<>>>><<<>>>><<<<>>>><<>>><<>>>><<>><<><<<<>><<<<>>><<<<>>><<>>><<<<>>><<<>>>><<>><<>\
    ><<<>>><><<>>><><<<><<<>>>><<<>>>><<<>><<>><<>><<>>>><<<><<<<>>>><>><<<<>>><<<>>><>>><<>>><<>>>><<<>><<<<>><>><<<<\
    ><<>>><<<<>>>><<<<>><><>>>><>>>><><<<>>>><<<<>>><<<<>>>><<<<><<>>><<>>>><>>>><<<<>>><>>><<>>><<<><<>><>>>><<>><>>>\
    <>><>>><<<<><<<>>>><<>>>><>>><>>><>>>><<<><<<<>>><<<>>>><<<><>>><<<>><>><>><<<>><<<>>><<<>>>><>><><>>><<<><><<<<><\
    <>><<<<>>><<<>><<<<>>><<<>>><<<<>>><<<>>><<<>>><<<>>>><<<<>>>><<<<><<<>>><<><<>>><<<<>>>><<>><<<<><>><<<<>><>><<<>\
    >><<>>>><>><>>><<<>><<<><<<>><<<>><<<<>>><<<<>>><<<<>>><<<>>>><<<>><<<<>><<><<<>>><>>>><<>>><<<<><<>>><>>>><<<<>><\
    <>>><<><>>><>>><<<<>>><<><<<><>>><<<>>><<>>><<<<><<><<<<>><<<<><<<>>><>><<>>>><<<>>>><<<<><<><<<<><<<<>>><>><<<><<\
    >>>><<<<>>>><<<<>>><<<>>><<<>>><<<<><<<>>>><<<<>><<<>><<<<>><><>><>><><<<>><<<>>><>>>><<<>><<<<>><><>>>><<<<>>><<<\
    <>>>><<<>><<<>>>><<<>>><<<>>><<<>><<>>>><><<<><<>>>><<<>><<>><<>>>><>>>><>>>><<>>>><<><<<>>>><<<>><<>>><<><<<<>>><\
    <<<>><<<>><>><<<>>>><<<><<>>>><<<><<>><<>>><<<>>><<><>>><<<>>><<<>>><>>>><<<<><<<<>>><>>>><<<>>><<><<>><<<<>>>><<<\
    <><<>>>><<<>>><<<><<>>>><<<<><>><<<>>>><<<<>>>><<><>>><<<>><<<>>>><<>>>><<>>>><<>>><<>><<>>><<<<><<<>>>><>><<<<>><\
    ><<>>><>><<<<><>><<>><><><>>><<>>>><>>>><<<<>><<<>>><<<<>>>><<<<>>><<<>>>><<>>><<>>><><>>>><<>><><<>>><<>>>><<<>>>\
    <<<<>>><<<>>><<>>><<<<>><<<<>><<<<>><<<>><<<<>>>><<<>>>><<<<><<<<>>>><>>><<<<>>>><<<>>><<<<><<<<><<><<<<>>>><>>>><\
    >>><<<>>>><<><<<><<<<>><<<<>>><<>>>><<>><<<>>>><>>><>>>><>>><<>><<<>>>><<<>>><<>>>><>>><<<>>><<>><<>>><<<>>>><<>>>\
    ><<><<<>>><<><<<>><<<<><<<>><<><>><<<>>><<>>><<<>>><<<<><<<<><<<<><><>>>><<<>><>>><>>><<<>>>><>>><>><<<<>>><<<><<<\
    >>>><>>><<<<><<<>>>><<<<>>>><<<><>><<<>><<<<>>><<<<>><<<<>><><<<<>>>><>>>><<>><<<<><<<><<<>>><<<>><<><<><>>><<<<>>\
    <<>>><<>>>><<<<><<<<><<<<>>><<>><<<<>>>><<<<>>>><<<>>>><<<<>><<<>><<<<>>>><<<<><<>><<<>>>><<><<<>><<<><<><<<<>>><>\
    >>><<>>>><<<<>>>><<>><<<>>><>><<>>>><<<>>>><>>><<<<>>>><>>>><<>><><<<<>>><<>>><<>><<><>><<<>>>><>>>><>>>><><<<<>><\
    <<>>><<<>>>><<<<><<<<>>>><<>>><<>>>><<>>>><><<><<>>><<><<>>><<><>>>><<>><<<>><<><<<<>>><<<>>>><<<<><>><<>>><<<>>>>\
    <<>>>><<>>>><<<>>>><<<<>>>><<>>>><<<><><>><>>><<<><<<>>>><><<<<>>>><<<<>>>><<>>><>>>><>>><<<<>>><<<>>><<<><>><<<<>\
    <<<<>>><>>>><>><<<>>><<<><<<<>>>><<>>><<<<><<<<><>><<<>>>><<<<>>><>>><<<<>><<>><<<<>>>><>>><<<<>><<<<><<><><<<<>><\
    <<>><<<>><<<>><<<>>><><<>>>><<><<<<>><<<><<<<>>><<<<>><><>><>>>><<<<>>><<><<<>><<<><<>>><>><>><<<>>>><<>>>><>><<<<\
    >><<<<><<<<>>><<<<>><<<>>>><<<<>>>><<><>>><<<>>><<>><<>><><<><<>>>><<<><<>>><<><<>>><<>>>><<<<>><<>>><<<>>><<<>>>>\
    <<><>>><<>>><>>><<<><<<><<>>>><<<<>>><<<<>>><<<>>><<<<>><><>>>><<<>>>><>><>>><<<>>><<<<>><>>>><>>><<<><<<><<<<><<<\
    <>>><<<><<<><>>><<<<><<<<>><>><<>>><<<>>><<<<><<>>><<<>>>><>>>><<<>>><<<<>>><<<<><>>>><<>><<<><<<>>><<<><<<<>>>><<\
    >><<<>><>><<<<>>><>>><<<<><><<<<>>><<>>>><<>><>>>><<<>><>>><<<>><<<<>>>><<<>><<<<><<>>><>>>><<>><<<<>>>><>><<>><<<\
    <>>><<>>><>>>><><><><<<<>>><>>><>>>><<<>><<><<<><>><<<<>><>><<><<<>>>><><<<>>><<>>><<>><<<<><<<<><<<<>><<<>>>><><<\
    >>>><<>>><<><<<>><<<>><<<<>>>><>>>><<<<>>><<<<>>><<<<>><>>>><<>>>><>><><<>>>><<<><><<<<>><<>><<><<<<>>><<>><>><<<>\
    >>><<>><<>>>><<>>><<<<>>><<<<>>>><<<>>>><>>><>>>><<<<>>>><<<<><<<><<>>><<<>>><<><>>>><<><<<>>><<<<><<<>><<<<>><<>>\
    ><<><<<>>><>><>><<<>>><<>>>><><<>>><><<<>><<<<>>>><<>><><<<>>>><><<<<><<<>>>><<<<>><<>>>><>>>><<>><<<<><<<<>>><<>>\
    >><<>>><<>>>><<<>>>><<>><<><>><<<>><<<>>><<>>><<<<>>><<>><<<>><<<<>><<<><<>>><<<<>>><<<><<<<>><<>><<<<>><<>><<>>><\
    <<>>><<>>><<>><<<>><<<>>>><>>><>><<<<>>><<>>>><>>>><<<<>>>><>>><<<<>>><<<>>>><<<<>>>><<<>>><<>>>><<<>><>>>><<<<>><\
    <<>><><><<<><>>>><<<>><><>>>><<<>>><<<<>>>><<>>>><<>><<>>><<<<>>>><<<><>>>><>><<>>><<<>>><<<<><<<>><<<<><<<>>><>>>\
    <<>>><<><>>><<>>>><><<>>>><>>>><<<<><<<><><<>><<<>><<><<<>><<><<>><<<<>>>><<<>><<<<>>><<<<>>>><<<>><<<><<<>>>><<<>\
    >>><><><<><><>>><<<>>>><>>><<<>>><<<<>>><><<<>>>><>><>>><<><<><<>>>><<<<>>><<<<>>>><><><<>><<<<>>><<<<>>><<<<><<<>\
    ><<<>>><<<<>>><<>><<>>><<<<>><<<><<<<><<<>>><<>>><<<>>><>><";
}
