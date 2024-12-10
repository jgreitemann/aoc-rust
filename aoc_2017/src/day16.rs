use std::{num::ParseIntError, str::FromStr};

use aoc_companion::prelude::*;
use itertools::Itertools;

pub(crate) struct Door {
    moves: Vec<DanceMove>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DanceMove {
    Spin(usize),
    Exchange(usize, usize),
    Partner(char, char),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ParseError {
    #[error("encountered an empty dance move instruction")]
    EmptyMove,
    #[error("dance move {0:?} is not recognized")]
    UnrecognizedDanceMove(char),
    #[error("dance move expects two '/'-separated parameters, but no '/' was found")]
    MissingSlash,
    #[error("spin size is invalid")]
    InvalidSpin(#[source] ParseIntError),
    #[error("program index for Exchange instruction is invalid")]
    InvalidIndex(#[source] ParseIntError),
    #[error("program name for Partner instruction is invalid: expected a single lowercase letter, got {0:?}")]
    InvalidName(String),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum MoveError {
    #[error(
        "program with index {index} is out-of-range for the dance group of only {len} programs"
    )]
    ProgramIndexOutOfBounds { index: usize, len: usize },
    #[error("no program with name {name:?} found in the dance group")]
    ProgramNameNotFound { name: char },
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseError> {
        let moves = input.split(',').map(str::parse).try_collect()?;
        Ok(Door { moves })
    }

    fn part1(&self) -> Result<String, MoveError> {
        self.moves
            .iter()
            .try_fold(initial_sequence(16), apply_move)
            .map(|v| v.iter().join(""))
    }

    fn part2(&self) -> Result<String, MoveError> {
        perform_many_dances(16, 1_000_000_000, &self.moves).map(|v| v.iter().join(""))
    }
}

impl FromStr for DanceMove {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseError::EmptyMove);
        }

        let Some((descriminator, args)) = s.split_at_checked(1) else {
            return Err(ParseError::UnrecognizedDanceMove(s.chars().next().unwrap()));
        };

        let descriminator = descriminator.chars().next().unwrap();

        match descriminator {
            's' => Ok(DanceMove::Spin(
                args.parse().map_err(ParseError::InvalidSpin)?,
            )),
            'x' | 'p' => {
                if let Some((lhs, rhs)) = args.split_once('/') {
                    match descriminator {
                        'x' => Ok(DanceMove::Exchange(
                            lhs.parse().map_err(ParseError::InvalidIndex)?,
                            rhs.parse().map_err(ParseError::InvalidIndex)?,
                        )),
                        'p' => Ok(DanceMove::Partner(
                            to_program_name(lhs)?,
                            to_program_name(rhs)?,
                        )),
                        _ => unreachable!(),
                    }
                } else {
                    Err(ParseError::MissingSlash)
                }
            }
            _ => Err(ParseError::UnrecognizedDanceMove(descriminator)),
        }
    }
}

fn to_program_name(s: &str) -> Result<char, ParseError> {
    match s.chars().exactly_one() {
        Ok(c) if c.is_ascii_lowercase() => Ok(c),
        _ => Err(ParseError::InvalidName(s.to_string())),
    }
}

fn initial_sequence(len: usize) -> Vec<char> {
    ('a'..).take(len).collect()
}

fn apply_move(mut state: Vec<char>, m: &DanceMove) -> Result<Vec<char>, MoveError> {
    let check_index = |index: usize, state: &[char]| {
        if index < state.len() {
            Ok(index)
        } else {
            Err(MoveError::ProgramIndexOutOfBounds {
                index,
                len: state.len(),
            })
        }
    };
    let find_name = |name: char, state: &[char]| {
        state
            .iter()
            .position(|&x| x == name)
            .ok_or(MoveError::ProgramNameNotFound { name })
    };

    match *m {
        DanceMove::Spin(size) => state.rotate_right(size),
        DanceMove::Exchange(lhs, rhs) => {
            let lhs = check_index(lhs, &state)?;
            let rhs = check_index(rhs, &state)?;
            state.swap(lhs, rhs)
        }
        DanceMove::Partner(lhs, rhs) => {
            let lhs = find_name(lhs, &state)?;
            let rhs = find_name(rhs, &state)?;
            state.swap(lhs, rhs)
        }
    };

    Ok(state)
}

fn determine_cycle(len: usize, dance: &[DanceMove]) -> Result<usize, MoveError> {
    let initial = initial_sequence(len);
    let mut state = initial.clone();
    for i in 1.. {
        state = dance.iter().try_fold(state, apply_move)?;
        if state == initial {
            return Ok(i);
        }
    }
    unreachable!()
}

fn perform_many_dances(len: usize, n: usize, dance: &[DanceMove]) -> Result<Vec<char>, MoveError> {
    let cycle = determine_cycle(len, dance)?;

    dance
        .iter()
        .cycle()
        .take(dance.len() * (n % cycle))
        .try_fold(initial_sequence(len), apply_move)
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::assert_equal;
    use proptest::prelude::*;

    const EXAMPLE_INPUT: &str = "s1,x3/4,pe/b";
    const EXAMPLE_DANCE: &[DanceMove] = &[
        DanceMove::Spin(1),
        DanceMove::Exchange(3, 4),
        DanceMove::Partner('e', 'b'),
    ];

    #[test]
    fn parse_example_input() {
        assert_eq!(Door::parse(EXAMPLE_INPUT).unwrap().moves, EXAMPLE_DANCE);
    }

    fn arbitrary_valid_input() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                any::<usize>().prop_map(|n| format!("s{n}")),
                (any::<usize>(), any::<usize>()).prop_map(|(lhs, rhs)| format!("x{lhs}/{rhs}")),
                "p[a-z]/[a-z]",
            ],
            1..100,
        )
        .prop_map(|v| v.join(","))
    }

    proptest! {
        #[test]
        fn parsing_invalid_input_does_not_panic(input in r"\PC*") {
            let _ = Door::parse(&input);
        }

        #[test]
        fn valid_input_is_parsed_successfully(input in arbitrary_valid_input()) {
            Door::parse(&input).unwrap();
        }
    }

    #[test]
    fn initial_character_sequence() {
        assert_eq!(initial_sequence(16).iter().join(""), "abcdefghijklmnop");
    }

    #[test]
    fn apply_moves_one_by_one() {
        assert_equal(
            EXAMPLE_DANCE.iter().scan(initial_sequence(5), |state, m| {
                *state = apply_move(std::mem::take(state), m).unwrap();
                Some(state.iter().join(""))
            }),
            ["eabcd", "eabdc", "baedc"],
        );
    }

    #[test]
    fn example_cycle() {
        assert_eq!(determine_cycle(5, EXAMPLE_DANCE).unwrap(), 4);
    }
}
