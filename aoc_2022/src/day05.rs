use aoc_companion::prelude::*;

use itertools::Itertools;
use thiserror::Error;

use std::str::FromStr;

#[derive(Debug)]
pub struct Door {
    stacks: Stacks,
    instructions: Vec<Instruction>,
}

impl ParseInput<'_> for Door {
    type Error = ParseError;

    fn parse(input: &str) -> Result<Self, Self::Error> {
        let (stacks_input, instr_input) = input.split_once("\n\n").unwrap();
        Ok(Self {
            stacks: stacks_input.parse()?,
            instructions: instr_input.lines().map(str::parse).try_collect()?,
        })
    }
}

impl Part1 for Door {
    type Output = String;
    type Error = Error;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        let final_stacks = self.stacks.clone().apply_all(&self.instructions, true)?;
        final_stacks.solution()
    }
}

impl Part2 for Door {
    type Output = String;
    type Error = Error;

    fn part2(&self) -> Result<Self::Output, Self::Error> {
        let final_stacks = self.stacks.clone().apply_all(&self.instructions, false)?;
        final_stacks.solution()
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Stacks string is empty")]
    NoLines,
    #[error("Rows are not equal in length")]
    UnequalRows,
    #[error("The regex to parse instructions failed to match")]
    InstructionRegexDoesNotMatch,
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("The stack index in the instruction is out-of-bounds")]
    StackIndexOutOfBounds,
    #[error("There aren't enough crates to move as instructed")]
    NotEnoughCratesToMove,
    #[error("A stack is empty")]
    EmptyStack,
}

#[derive(Debug, Clone)]
struct Stacks(Vec<Vec<char>>);

impl Stacks {
    fn apply(
        mut self,
        &Instruction { amount, from, to }: &Instruction,
        rev: bool,
    ) -> Result<Stacks, Error> {
        if from == 0 || to == 0 {
            return Err(Error::StackIndexOutOfBounds);
        }
        let len = self
            .0
            .get(from - 1)
            .ok_or(Error::StackIndexOutOfBounds)?
            .len();
        let idx = len
            .checked_sub(amount)
            .ok_or(Error::NotEnoughCratesToMove)?;
        let top = self.0[from - 1].split_off(idx);
        let to_stack = self.0.get_mut(to - 1).ok_or(Error::StackIndexOutOfBounds)?;
        if rev {
            to_stack.extend(top.into_iter().rev());
        } else {
            to_stack.extend_from_slice(&top);
        }
        Ok(self)
    }

    fn apply_all(self, instructions: &[Instruction], rev: bool) -> Result<Stacks, Error> {
        instructions
            .iter()
            .try_fold(self, |stacks, instr| stacks.apply(instr, rev))
    }

    fn solution(&self) -> Result<String, Error> {
        self.0
            .iter()
            .map(|s| s.last().cloned().ok_or(Error::EmptyStack))
            .collect()
    }
}

impl FromStr for Stacks {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rows: Vec<Vec<char>> = s
            .lines()
            .map(|line| {
                line.chars()
                    .chunks(4)
                    .into_iter()
                    .map(|chunk| chunk.into_iter().nth(1).unwrap())
                    .collect()
            })
            .collect();

        let n_cols = rows.first().ok_or(ParseError::NoLines)?.len();

        let cols: Result<Vec<Vec<char>>, ParseError> = rows[..rows.len() - 1]
            .iter()
            .rev()
            .try_fold(vec![Vec::new(); n_cols], |mut cols, row| {
                for (i, &c) in row.iter().enumerate().filter(|&(_, &c)| c != ' ') {
                    cols.get_mut(i).ok_or(ParseError::UnequalRows)?.push(c);
                }
                Ok(cols)
            });

        Ok(Self(cols?))
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Instruction {
    amount: usize,
    from: usize,
    to: usize,
}

impl FromStr for Instruction {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re =
            regex::Regex::new(r"move (?P<amount>\d+) from (?P<from>\d+) to (?P<to>\d+)").unwrap();
        let captures = re
            .captures(s)
            .ok_or(ParseError::InstructionRegexDoesNotMatch)?;
        Ok(Instruction {
            amount: captures["amount"].parse()?,
            from: captures["from"].parse()?,
            to: captures["to"].parse()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::assert_equal;

    const STACKS_INPUT: &str = "    [D]    \n\
[N] [C]    \n\
[Z] [M] [P]\n\
1   2   3 ";

    const EXAMPLE_STACKS: &[&[char]] = &[&['Z', 'N'], &['M', 'C', 'D'], &['P']];
    const EXAMPLE_STACKS_AFTER_FIRST: &[&[char]] = &[&['Z', 'N', 'D'], &['M', 'C'], &['P']];
    const EXAMPLE_FINAL_REV_STACKS: &[&[char]] = &[&['C'], &['M'], &['P', 'D', 'N', 'Z']];
    const EXAMPLE_FINAL_NOREV_STACKS: &[&[char]] = &[&['M'], &['C'], &['P', 'Z', 'N', 'D']];

    const INSTRUCTIONS_INPUT: &str = r"move 1 from 2 to 1
move 3 from 1 to 3
move 2 from 2 to 1
move 1 from 1 to 2";

    const EXAMPLE_INSTRUCTIONS: &[Instruction] = &[
        Instruction {
            amount: 1,
            from: 2,
            to: 1,
        },
        Instruction {
            amount: 3,
            from: 1,
            to: 3,
        },
        Instruction {
            amount: 2,
            from: 2,
            to: 1,
        },
        Instruction {
            amount: 1,
            from: 1,
            to: 2,
        },
    ];

    fn assert_stacks_eq(stacks: &Stacks, expected: &[&[char]]) {
        assert_equal(
            stacks.0.iter().cloned(),
            expected.iter().map(|col| col.to_vec()),
        );
    }

    #[test]
    fn stacks_are_parsed() {
        let stacks: Stacks = STACKS_INPUT.parse().unwrap();
        assert_stacks_eq(&stacks, EXAMPLE_STACKS);
    }

    #[test]
    fn instructions_are_parsed() {
        let instructions: Vec<Instruction> = INSTRUCTIONS_INPUT
            .lines()
            .map(str::parse)
            .try_collect()
            .unwrap();
        assert_eq!(instructions.as_slice(), EXAMPLE_INSTRUCTIONS);
    }

    #[test]
    fn stacks_after_first_instruction() {
        let stacks: Stacks = STACKS_INPUT.parse().unwrap();
        assert_stacks_eq(
            &stacks
                .apply(EXAMPLE_INSTRUCTIONS.first().unwrap(), true)
                .unwrap(),
            EXAMPLE_STACKS_AFTER_FIRST,
        );
    }

    #[test]
    fn final_rev_stacks() {
        let final_stacks = STACKS_INPUT
            .parse::<Stacks>()
            .unwrap()
            .apply_all(EXAMPLE_INSTRUCTIONS, true)
            .unwrap();
        assert_stacks_eq(&final_stacks, EXAMPLE_FINAL_REV_STACKS);
    }

    #[test]
    fn final_norev_stacks() {
        let final_stacks = STACKS_INPUT
            .parse::<Stacks>()
            .unwrap()
            .apply_all(EXAMPLE_INSTRUCTIONS, false)
            .unwrap();
        assert_stacks_eq(&final_stacks, EXAMPLE_FINAL_NOREV_STACKS);
    }
}
