use aoc_companion::prelude::*;

use itertools::Itertools;
use thiserror::Error;

pub struct Door<'input> {
    program: Vec<Instruction<'input>>,
}

impl<'input> ParseInput<'input> for Door<'input> {
    type Error = ParseError;

    fn parse(input: &'input str) -> Result<Self, Self::Error> {
        parse_input(input).map(|program| Self { program })
    }
}

impl Part1 for Door<'_> {
    type Output = i32;
    type Error = ExecutionError;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        largest_register_value(&execute_program(&self.program))
            .ok_or(ExecutionError::RegistersEmpty)
    }
}

impl Part2 for Door<'_> {
    type Output = i32;
    type Error = ExecutionError;

    fn part2(&self) -> Result<Self::Output, Self::Error> {
        largest_intermediate_register_value(&self.program)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Register<'input>(&'input str);

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Registers<'input>(std::collections::HashMap<Register<'input>, i32>);

impl<'input> Registers<'input> {
    fn execute(mut self, instr: &Instruction<'input>) -> Self {
        if self.eval(&instr.condition) {
            let target = self.0.entry(instr.target).or_default();
            match instr.operation {
                Operation::Increase(amount) => *target += amount,
                Operation::Decrease(amount) => *target -= amount,
            }
        }
        self
    }

    fn eval(&self, cond: &Condition<'input>) -> bool {
        let lhs = self.0.get(&cond.register).unwrap_or(&0);
        cond.cmp.as_fn()(lhs, &cond.value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Operation {
    Increase(i32),
    Decrease(i32),
}

#[derive(Debug, PartialEq, Eq)]
enum Comparison {
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
    Equal,
    NotEqual,
}

impl Comparison {
    fn as_fn(&self) -> fn(&i32, &i32) -> bool {
        use Comparison::*;
        match self {
            LessThan => PartialOrd::lt,
            GreaterThan => PartialOrd::gt,
            LessEqual => PartialOrd::le,
            GreaterEqual => PartialOrd::ge,
            Equal => PartialEq::eq,
            NotEqual => PartialEq::ne,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Condition<'input> {
    register: Register<'input>,
    cmp: Comparison,
    value: i32,
}

#[derive(Debug, PartialEq, Eq)]
struct Instruction<'input> {
    target: Register<'input>,
    operation: Operation,
    condition: Condition<'input>,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Line does not match the regular expression: {line:?}")]
    LineDoesNotMatch { line: String },
}

#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("The program does not contain any instructions")]
    ProgramEmpty,
    #[error("The registers are empty, because none of the instructions met its condition")]
    RegistersEmpty,
}

fn parse_input(input: &str) -> Result<Vec<Instruction>, ParseError> {
    let re = regex::Regex::new(r"^(?P<target>[a-z]+) (?P<op>inc|dec) (?P<amount>-?[0-9]+) if (?P<register>[a-z]+) (?P<cmp><|>|<=|>=|==|!=) (?P<value>-?[0-9]+)$").unwrap();

    input
        .lines()
        .map(|line| {
            re.captures(line)
                .ok_or_else(|| ParseError::LineDoesNotMatch {
                    line: line.to_string(),
                })
        })
        .map_ok(|caps| {
            let amount = caps.name("amount").unwrap().as_str().parse().unwrap();
            let value = caps.name("value").unwrap().as_str().parse().unwrap();
            Instruction {
                target: Register(caps.name("target").unwrap().as_str()),
                operation: match caps.name("op").unwrap().as_str() {
                    "inc" => Operation::Increase(amount),
                    "dec" => Operation::Decrease(amount),
                    _ => unreachable!(),
                },
                condition: Condition {
                    register: Register(caps.name("register").unwrap().as_str()),
                    cmp: match caps.name("cmp").unwrap().as_str() {
                        "<" => Comparison::LessThan,
                        ">" => Comparison::GreaterThan,
                        "<=" => Comparison::LessEqual,
                        ">=" => Comparison::GreaterEqual,
                        "==" => Comparison::Equal,
                        "!=" => Comparison::NotEqual,
                        _ => unreachable!(),
                    },
                    value,
                },
            }
        })
        .collect()
}

fn execute_program<'input>(instructions: &[Instruction<'input>]) -> Registers<'input> {
    instructions
        .iter()
        .fold(Registers::default(), Registers::execute)
}

fn largest_intermediate_register_value(
    instructions: &[Instruction],
) -> Result<i32, ExecutionError> {
    instructions
        .iter()
        .scan(Registers::default(), |registers, instr| {
            *registers = std::mem::take(registers).execute(instr);
            Some(largest_register_value(registers))
        })
        .max()
        .ok_or(ExecutionError::ProgramEmpty)
        .and_then(|max| max.ok_or(ExecutionError::RegistersEmpty))
}

fn largest_register_value(registers: &Registers) -> Option<i32> {
    registers.0.values().max().copied()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use assert_matches::assert_matches;
    use itertools::assert_equal;

    use super::*;

    const EXAMPLE_INSTRUCTIONS: &[Instruction] = &[
        Instruction {
            target: Register("b"),
            operation: Operation::Increase(5),
            condition: Condition {
                register: Register("a"),
                cmp: Comparison::GreaterThan,
                value: 1,
            },
        },
        Instruction {
            target: Register("a"),
            operation: Operation::Increase(1),
            condition: Condition {
                register: Register("b"),
                cmp: Comparison::LessThan,
                value: 5,
            },
        },
        Instruction {
            target: Register("c"),
            operation: Operation::Decrease(-10),
            condition: Condition {
                register: Register("a"),
                cmp: Comparison::GreaterEqual,
                value: 1,
            },
        },
        Instruction {
            target: Register("c"),
            operation: Operation::Increase(-20),
            condition: Condition {
                register: Register("c"),
                cmp: Comparison::Equal,
                value: 10,
            },
        },
    ];

    #[test]
    fn example_input_is_parsed() {
        const EXAMPLE_INPUT: &str = r"b inc 5 if a > 1
a inc 1 if b < 5
c dec -10 if a >= 1
c inc -20 if c == 10
";
        assert_eq!(
            parse_input(EXAMPLE_INPUT).unwrap().as_slice(),
            EXAMPLE_INSTRUCTIONS,
        );
    }

    #[test]
    fn intermediate_registers() {
        assert_equal(
            EXAMPLE_INSTRUCTIONS
                .into_iter()
                .scan(Registers::default(), |registers, instr| {
                    *registers = std::mem::take(registers).execute(instr);
                    Some(registers.clone())
                }),
            [
                Registers::default(),
                Registers(HashMap::from([(Register("a"), 1)])),
                Registers(HashMap::from([(Register("a"), 1), (Register("c"), 10)])),
                Registers(HashMap::from([(Register("a"), 1), (Register("c"), -10)])),
            ],
        );
    }

    #[test]
    fn final_program_registers() {
        assert_eq!(
            execute_program(EXAMPLE_INSTRUCTIONS),
            Registers(HashMap::from([(Register("a"), 1), (Register("c"), -10)]))
        );
    }

    #[test]
    fn largest_register_value_is_determined() {
        assert_matches!(largest_register_value(&Registers::default()), None);
        assert_matches!(
            largest_register_value(&Registers(HashMap::from([
                (Register("a"), 1),
                (Register("c"), -10)
            ]))),
            Some(1)
        );
    }

    #[test]
    fn largest_intermediate_register_value_is_determined() {
        assert_matches!(largest_intermediate_register_value(EXAMPLE_INSTRUCTIONS), Ok(10));
    }
}
