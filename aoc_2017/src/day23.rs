use std::{collections::HashMap, str::FromStr};

use anyhow::anyhow;
use aoc_companion::prelude::*;
use enum_map::{Enum, EnumMap};
use itertools::Itertools;

use crate::day18::{Operand, ParseError, Register};

pub(crate) struct Door {
    program: Vec<Instruction>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseError> {
        let program = input.lines().map(str::parse).try_collect()?;
        Ok(Self { program })
    }

    fn part1(&self) -> usize {
        let profile = run(&self.program, &mut HashMap::new());
        profile[Op::Mul]
    }

    fn part2(&self) -> Result<usize> {
        if self.program[8..] == PROGRAM[8..] {
            let mut registers = HashMap::from([(Register(b'a'), 1)]);
            run(&self.program[0..8], &mut registers);
            Ok(reverse_engineered_function(
                registers[&Register(b'b')] as u64,
                registers[&Register(b'c')] as u64,
            ))
        } else {
            Err(anyhow!(
                "input program does not match reverse-engineered program"
            ))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
enum Op {
    Set,
    Sub,
    Mul,
    Jnz,
}

impl FromStr for Op {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "set" => Ok(Op::Set),
            "sub" => Ok(Op::Sub),
            "mul" => Ok(Op::Mul),
            "jnz" => Ok(Op::Jnz),
            _ => Err(ParseError::InvalidInstruction(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Instruction {
    op: Op,
    lhs: Operand,
    rhs: Operand,
}

impl FromStr for Instruction {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (op, args) = s.split_once(' ').ok_or(ParseError::MissingToken)?;
        let (lhs, rhs) = args.split_once(' ').ok_or(ParseError::MissingToken)?;
        Ok(Instruction {
            op: op.parse()?,
            lhs: lhs.parse()?,
            rhs: rhs.parse()?,
        })
    }
}

type Profile = EnumMap<Op, usize>;

fn run(program: &[Instruction], registers: &mut HashMap<Register, i64>) -> Profile {
    let mut pc = 0;
    let mut profile = Profile::default();
    while let Some(instruction) = pc.try_into().ok().and_then(|pc: usize| program.get(pc)) {
        profile[instruction.op] += 1;

        match instruction {
            Instruction {
                op: Op::Set,
                lhs: Operand::Register(reg),
                rhs: arg,
            } => {
                *reg.access(registers).or_default() = arg.fetch(registers);
            }
            Instruction {
                op: Op::Sub,
                lhs: Operand::Register(reg),
                rhs: arg,
            } => {
                *reg.access(registers).or_default() -= arg.fetch(registers);
            }
            Instruction {
                op: Op::Mul,
                lhs: Operand::Register(reg),
                rhs: arg,
            } => {
                *reg.access(registers).or_default() *= arg.fetch(registers);
            }
            Instruction {
                op: Op::Jnz,
                lhs,
                rhs,
            } if lhs.fetch(registers) != 0 => {
                pc += rhs.fetch(registers);
                continue;
            }
            _ => {}
        }
        pc += 1;
    }

    profile
}

fn reverse_engineered_function(b: u64, c: u64) -> usize {
    use primes::PrimeSet;
    let mut sieve = primes::Sieve::new();
    (b..=c).step_by(17).filter(|&x| !sieve.is_prime(x)).count()
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use enum_map::enum_map;
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn profile_in_debug_mode() {
        let profile = run(PROGRAM, &mut HashMap::new());
        assert_eq!(
            profile,
            enum_map! {
                Op::Set => 16751,
                Op::Sub => 25027,
                Op::Mul => 8281,
                Op::Jnz => 16658,
            }
        );
    }

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(8))]
        #[test]
        fn program_counts_non_prime_multiple_of_17_in_range(start in 2u64..1000, mult in 1u64..25) {
            let end = start + mult * 17;
            let mut registers = HashMap::from([(Register(b'b'), start as i64), (Register(b'c'), end as i64)]);
            run(&PROGRAM[8..], &mut registers);
            assert_eq!(
                registers[&Register(b'h')] as usize,
                reverse_engineered_function(start, end)
            );
        }

    }
}

const PROGRAM: &[Instruction] = &[
    Instruction {
        op: Op::Set,
        lhs: Operand::Register(Register(b'b')),
        rhs: Operand::Immediate(93),
    },
    Instruction {
        op: Op::Set,
        lhs: Operand::Register(Register(b'c')),
        rhs: Operand::Register(Register(b'b')),
    },
    Instruction {
        op: Op::Jnz,
        lhs: Operand::Register(Register(b'a')),
        rhs: Operand::Immediate(2),
    },
    Instruction {
        op: Op::Jnz,
        lhs: Operand::Immediate(1),
        rhs: Operand::Immediate(5),
    },
    Instruction {
        op: Op::Mul,
        lhs: Operand::Register(Register(b'b')),
        rhs: Operand::Immediate(100),
    },
    Instruction {
        op: Op::Sub,
        lhs: Operand::Register(Register(b'b')),
        rhs: Operand::Immediate(-100000),
    },
    Instruction {
        op: Op::Set,
        lhs: Operand::Register(Register(b'c')),
        rhs: Operand::Register(Register(b'b')),
    },
    Instruction {
        op: Op::Sub,
        lhs: Operand::Register(Register(b'c')),
        rhs: Operand::Immediate(-17000),
    },
    Instruction {
        op: Op::Set,
        lhs: Operand::Register(Register(b'f')),
        rhs: Operand::Immediate(1),
    },
    Instruction {
        op: Op::Set,
        lhs: Operand::Register(Register(b'd')),
        rhs: Operand::Immediate(2),
    },
    Instruction {
        op: Op::Set,
        lhs: Operand::Register(Register(b'e')),
        rhs: Operand::Immediate(2),
    },
    Instruction {
        op: Op::Set,
        lhs: Operand::Register(Register(b'g')),
        rhs: Operand::Register(Register(b'd')),
    },
    Instruction {
        op: Op::Mul,
        lhs: Operand::Register(Register(b'g')),
        rhs: Operand::Register(Register(b'e')),
    },
    Instruction {
        op: Op::Sub,
        lhs: Operand::Register(Register(b'g')),
        rhs: Operand::Register(Register(b'b')),
    },
    Instruction {
        op: Op::Jnz,
        lhs: Operand::Register(Register(b'g')),
        rhs: Operand::Immediate(2),
    },
    Instruction {
        op: Op::Set,
        lhs: Operand::Register(Register(b'f')),
        rhs: Operand::Immediate(0),
    },
    Instruction {
        op: Op::Sub,
        lhs: Operand::Register(Register(b'e')),
        rhs: Operand::Immediate(-1),
    },
    Instruction {
        op: Op::Set,
        lhs: Operand::Register(Register(b'g')),
        rhs: Operand::Register(Register(b'e')),
    },
    Instruction {
        op: Op::Sub,
        lhs: Operand::Register(Register(b'g')),
        rhs: Operand::Register(Register(b'b')),
    },
    Instruction {
        op: Op::Jnz,
        lhs: Operand::Register(Register(b'g')),
        rhs: Operand::Immediate(-8),
    },
    Instruction {
        op: Op::Sub,
        lhs: Operand::Register(Register(b'd')),
        rhs: Operand::Immediate(-1),
    },
    Instruction {
        op: Op::Set,
        lhs: Operand::Register(Register(b'g')),
        rhs: Operand::Register(Register(b'd')),
    },
    Instruction {
        op: Op::Sub,
        lhs: Operand::Register(Register(b'g')),
        rhs: Operand::Register(Register(b'b')),
    },
    Instruction {
        op: Op::Jnz,
        lhs: Operand::Register(Register(b'g')),
        rhs: Operand::Immediate(-13),
    },
    Instruction {
        op: Op::Jnz,
        lhs: Operand::Register(Register(b'f')),
        rhs: Operand::Immediate(2),
    },
    Instruction {
        op: Op::Sub,
        lhs: Operand::Register(Register(b'h')),
        rhs: Operand::Immediate(-1),
    },
    Instruction {
        op: Op::Set,
        lhs: Operand::Register(Register(b'g')),
        rhs: Operand::Register(Register(b'b')),
    },
    Instruction {
        op: Op::Sub,
        lhs: Operand::Register(Register(b'g')),
        rhs: Operand::Register(Register(b'c')),
    },
    Instruction {
        op: Op::Jnz,
        lhs: Operand::Register(Register(b'g')),
        rhs: Operand::Immediate(2),
    },
    Instruction {
        op: Op::Jnz,
        lhs: Operand::Immediate(1),
        rhs: Operand::Immediate(3),
    },
    Instruction {
        op: Op::Sub,
        lhs: Operand::Register(Register(b'b')),
        rhs: Operand::Immediate(-17),
    },
    Instruction {
        op: Op::Jnz,
        lhs: Operand::Immediate(1),
        rhs: Operand::Immediate(-23),
    },
];
