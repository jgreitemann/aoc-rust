use aoc_companion::prelude::*;

use itertools::Itertools;
use thiserror::Error;

use std::str::FromStr;

pub(crate) struct Door {
    program: Vec<Instruction>,
}

impl<'input> ParseInput<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseError> {
        parse_input(input).map(|program| Self { program })
    }
}

impl Part1 for Door {
    fn part1(&self) -> isize {
        relevant_signal_strengths(execute(&self.program)).sum()
    }
}

impl Part2 for Door {
    fn part2(&self) -> Result<String, ReadError> {
        read_screen(&render(execute(&self.program)))
    }
}

#[derive(Debug, Error)]
pub(crate) enum ParseError {
    #[error("Empty line")]
    EmptyLine,
    #[error("Encountered an invalid instruction: {0:?}")]
    InvalidInstruction(String),
    #[error("Missing the argument to a {instruction} instruction")]
    MissingArgument { instruction: String },
    #[error("Could not parse instruction argument: {0}")]
    InvalidIncrement(#[from] std::num::ParseIntError),
}

#[derive(Debug, Error)]
pub(crate) enum ReadError {
    #[error("Human help is needed in reading the displayed string:\n{0}")]
    NeedToRead(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Instruction {
    Noop,
    Addx(isize),
}

impl FromStr for Instruction {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = s.split_whitespace();
        let instr_str = tokens.next();
        let arg_str = tokens.next();
        match instr_str {
            Some("noop") => Ok(Instruction::Noop),
            Some("addx") => Ok(Instruction::Addx(
                arg_str
                    .ok_or(ParseError::MissingArgument {
                        instruction: "addx".to_owned(),
                    })
                    .and_then(|a| a.parse::<isize>().map_err(|e| e.into()))?,
            )),
            Some(instr) => Err(ParseError::InvalidInstruction(instr.to_owned())),
            None => Err(ParseError::EmptyLine),
        }
    }
}

fn parse_input(input: &str) -> Result<Vec<Instruction>, ParseError> {
    input.lines().map(str::parse).collect()
}

fn execute(program: &[Instruction]) -> impl Iterator<Item = isize> + '_ {
    use genawaiter::{rc::gen, yield_};
    let mut x = 1;
    gen!({
        for instr in program {
            match instr {
                Instruction::Noop => yield_!(x),
                Instruction::Addx(y) => {
                    yield_!(x);
                    yield_!(x);
                    x += y;
                }
            }
        }
        yield_!(x)
    })
    .into_iter()
}

fn relevant_signal_strengths(values: impl Iterator<Item = isize>) -> impl Iterator<Item = isize> {
    values
        .zip(1isize..)
        .skip(19)
        .step_by(40)
        .take(6)
        .map(|(val, cycle)| val * cycle)
}

#[allow(unstable_name_collisions)]
fn render(values: impl Iterator<Item = isize>) -> String {
    const WIDTH: usize = 40;
    const HEIGHT: usize = 6;

    let continuous: String = values
        .zip(0..(WIDTH * HEIGHT))
        .map(|(sprite_pos, idx)| {
            if (sprite_pos - 1..=sprite_pos + 1).contains(&((idx % WIDTH) as isize)) {
                '#'
            } else {
                '.'
            }
        })
        .collect();
    let bytes: Vec<_> = continuous
        .as_bytes()
        .chunks(WIDTH)
        .intersperse(b"\n")
        .flatten()
        .copied()
        .collect();
    String::from_utf8(bytes).unwrap()
}

fn read_screen(screen: &str) -> Result<String, ReadError> {
    const FJUBULRZ_SCREEN: &str = "####...##.#..#.###..#..#.#....###..####.\n\
                                   #.......#.#..#.#..#.#..#.#....#..#....#.\n\
                                   ###.....#.#..#.###..#..#.#....#..#...#..\n\
                                   #.......#.#..#.#..#.#..#.#....###...#...\n\
                                   #....#..#.#..#.#..#.#..#.#....#.#..#....\n\
                                   #.....##...##..###...##..####.#..#.####.";

    match screen {
        FJUBULRZ_SCREEN => Ok("FJUBULRZ".to_owned()),
        _ => Err(ReadError::NeedToRead(screen.to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use super::Instruction::*;
    use super::*;

    use itertools::assert_equal;

    const SMALL_EXAMPLE_INPUT: &str = r"noop
addx 3
addx -5";

    const SMALL_EXAMPLE_INSTRUCTIONS: &[Instruction] = &[Noop, Addx(3), Addx(-5)];

    #[test]
    fn input_is_parsed() {
        assert_eq!(
            parse_input(SMALL_EXAMPLE_INPUT).unwrap(),
            SMALL_EXAMPLE_INSTRUCTIONS
        );
        assert_eq!(parse_input(EXAMPLE_INPUT).unwrap(), EXAMPLE_INSTRUCTIONS);
    }

    #[test]
    fn execution_produces_correct_sequence() {
        assert_equal(execute(SMALL_EXAMPLE_INSTRUCTIONS), [1, 1, 1, 4, 4, -1]);
    }

    #[test]
    fn relevant_signal_strengths_are_produced() {
        assert_equal(
            relevant_signal_strengths(execute(EXAMPLE_INSTRUCTIONS)),
            [420, 1140, 1800, 2940, 2880, 3960],
        )
    }

    #[test]
    fn crt_image_is_rendered() {
        assert_eq!(render(execute(EXAMPLE_INSTRUCTIONS)), EXAMPLE_RENDER);
    }

    const EXAMPLE_INPUT: &str = r"addx 15
addx -11
addx 6
addx -3
addx 5
addx -1
addx -8
addx 13
addx 4
noop
addx -1
addx 5
addx -1
addx 5
addx -1
addx 5
addx -1
addx 5
addx -1
addx -35
addx 1
addx 24
addx -19
addx 1
addx 16
addx -11
noop
noop
addx 21
addx -15
noop
noop
addx -3
addx 9
addx 1
addx -3
addx 8
addx 1
addx 5
noop
noop
noop
noop
noop
addx -36
noop
addx 1
addx 7
noop
noop
noop
addx 2
addx 6
noop
noop
noop
noop
noop
addx 1
noop
noop
addx 7
addx 1
noop
addx -13
addx 13
addx 7
noop
addx 1
addx -33
noop
noop
noop
addx 2
noop
noop
noop
addx 8
noop
addx -1
addx 2
addx 1
noop
addx 17
addx -9
addx 1
addx 1
addx -3
addx 11
noop
noop
addx 1
noop
addx 1
noop
noop
addx -13
addx -19
addx 1
addx 3
addx 26
addx -30
addx 12
addx -1
addx 3
addx 1
noop
noop
noop
addx -9
addx 18
addx 1
addx 2
noop
noop
addx 9
noop
noop
noop
addx -1
addx 2
addx -37
addx 1
addx 3
noop
addx 15
addx -21
addx 22
addx -6
addx 1
noop
addx 2
addx 1
noop
addx -10
noop
noop
addx 20
addx 1
addx 2
addx 2
addx -6
addx -11
noop
noop
noop";

    const EXAMPLE_INSTRUCTIONS: &[Instruction] = &[
        Addx(15),
        Addx(-11),
        Addx(6),
        Addx(-3),
        Addx(5),
        Addx(-1),
        Addx(-8),
        Addx(13),
        Addx(4),
        Noop,
        Addx(-1),
        Addx(5),
        Addx(-1),
        Addx(5),
        Addx(-1),
        Addx(5),
        Addx(-1),
        Addx(5),
        Addx(-1),
        Addx(-35),
        Addx(1),
        Addx(24),
        Addx(-19),
        Addx(1),
        Addx(16),
        Addx(-11),
        Noop,
        Noop,
        Addx(21),
        Addx(-15),
        Noop,
        Noop,
        Addx(-3),
        Addx(9),
        Addx(1),
        Addx(-3),
        Addx(8),
        Addx(1),
        Addx(5),
        Noop,
        Noop,
        Noop,
        Noop,
        Noop,
        Addx(-36),
        Noop,
        Addx(1),
        Addx(7),
        Noop,
        Noop,
        Noop,
        Addx(2),
        Addx(6),
        Noop,
        Noop,
        Noop,
        Noop,
        Noop,
        Addx(1),
        Noop,
        Noop,
        Addx(7),
        Addx(1),
        Noop,
        Addx(-13),
        Addx(13),
        Addx(7),
        Noop,
        Addx(1),
        Addx(-33),
        Noop,
        Noop,
        Noop,
        Addx(2),
        Noop,
        Noop,
        Noop,
        Addx(8),
        Noop,
        Addx(-1),
        Addx(2),
        Addx(1),
        Noop,
        Addx(17),
        Addx(-9),
        Addx(1),
        Addx(1),
        Addx(-3),
        Addx(11),
        Noop,
        Noop,
        Addx(1),
        Noop,
        Addx(1),
        Noop,
        Noop,
        Addx(-13),
        Addx(-19),
        Addx(1),
        Addx(3),
        Addx(26),
        Addx(-30),
        Addx(12),
        Addx(-1),
        Addx(3),
        Addx(1),
        Noop,
        Noop,
        Noop,
        Addx(-9),
        Addx(18),
        Addx(1),
        Addx(2),
        Noop,
        Noop,
        Addx(9),
        Noop,
        Noop,
        Noop,
        Addx(-1),
        Addx(2),
        Addx(-37),
        Addx(1),
        Addx(3),
        Noop,
        Addx(15),
        Addx(-21),
        Addx(22),
        Addx(-6),
        Addx(1),
        Noop,
        Addx(2),
        Addx(1),
        Noop,
        Addx(-10),
        Noop,
        Noop,
        Addx(20),
        Addx(1),
        Addx(2),
        Addx(2),
        Addx(-6),
        Addx(-11),
        Noop,
        Noop,
        Noop,
    ];

    const EXAMPLE_RENDER: &str = r"##..##..##..##..##..##..##..##..##..##..
###...###...###...###...###...###...###.
####....####....####....####....####....
#####.....#####.....#####.....#####.....
######......######......######......####
#######.......#######.......#######.....";
}
