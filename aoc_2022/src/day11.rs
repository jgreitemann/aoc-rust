use aoc_companion::prelude::*;

use itertools::Itertools;
use thiserror::Error;

use std::collections::HashMap;
use std::str::FromStr;

pub(crate) struct Door {
    monkeys: Vec<Monkey>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseError> {
        parse_input(input).map(|monkeys| Self { monkeys })
    }

    fn part1(&self) -> usize {
        let mut game = Game::<u64>::from(&self.monkeys, Operation::DivByThree);
        game.play_rounds(20);
        game.monkey_business()
    }

    fn part2(&self) -> usize {
        let mut game = Game::<ModuloTableWorryLevel>::from(&self.monkeys, Operation::NoOp);
        game.play_rounds(10000);
        game.monkey_business()
    }
}

#[derive(Debug, Error)]
pub(crate) enum ParseError {
    #[error("Incomplete monkey input: not enough lines")]
    NotEnoughLinesForMonkey,
    #[error("Could not find monkey's starting items on line {0:?}")]
    NoStartingItems(String),
    #[error("Could not find monkey's operation on line {0:?}")]
    NoOperation(String),
    #[error("Could not find monkey's test divisor on line {0:?}")]
    NoTestDivisor(String),
    #[error("Could not find monkey's {0} target on line {1:?}")]
    NoTarget(bool, String),
    #[error("Failed to tokenize operation: {0:?}")]
    OperationTokenization(String),
    #[error("Unknown operator: {0:?}")]
    UnknownOperator(String),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Monkey {
    starting_items: Vec<u64>,
    operation: Operation,
    test_divisor: u64,
    true_target: usize,
    false_target: usize,
}

impl Monkey {
    fn target<W: WorryLevel>(&self, worry: &W) -> usize {
        if worry.divisible_by(&self.test_divisor) {
            self.true_target
        } else {
            self.false_target
        }
    }
}

impl FromStr for Monkey {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let &[
            starting_items_line,
            operation_line,
            test_line,
            true_line,
            false_line,
            ..,
        ] = &s.lines().collect_vec()[1..]
        {
            Ok(Monkey {
                starting_items: starting_items_line
                    .trim()
                    .strip_prefix("Starting items: ")
                    .ok_or_else(|| ParseError::NoStartingItems(starting_items_line.to_owned()))?
                    .split(", ")
                    .map(str::parse)
                    .try_collect()?,
                operation: operation_line
                    .trim()
                    .strip_prefix("Operation: new = ")
                    .ok_or_else(|| ParseError::NoOperation(operation_line.to_owned()))?
                    .parse()?,
                test_divisor: test_line
                    .trim()
                    .strip_prefix("Test: divisible by ")
                    .ok_or_else(|| ParseError::NoTestDivisor(test_line.to_owned()))?
                    .parse()?,
                true_target: true_line
                    .trim()
                    .strip_prefix("If true: throw to monkey ")
                    .ok_or_else(|| ParseError::NoTarget(true, true_line.to_owned()))?
                    .parse()?,
                false_target: false_line
                    .trim()
                    .strip_prefix("If false: throw to monkey ")
                    .ok_or_else(|| ParseError::NoTarget(false, false_line.to_owned()))?
                    .parse()?,
            })
        } else {
            Err(ParseError::NotEnoughLinesForMonkey)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Operation {
    NoOp,
    Add(u64),
    Mul(u64),
    MulBySelf,
    DivByThree,
}

impl FromStr for Operation {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((op_str, rhs_str)) =
            s.strip_prefix("old ").and_then(|rest| rest.split_once(' '))
        {
            match op_str {
                "+" => Ok(Operation::Add(rhs_str.parse()?)),
                "*" => Ok(if rhs_str == "old" {
                    Operation::MulBySelf
                } else {
                    Operation::Mul(rhs_str.parse()?)
                }),
                _ => Err(ParseError::UnknownOperator(op_str.to_owned())),
            }
        } else {
            Err(ParseError::OperationTokenization(s.to_owned()))
        }
    }
}

fn parse_input(input: &str) -> Result<Vec<Monkey>, ParseError> {
    input.split("\n\n").map(str::parse).try_collect()
}

trait WorryLevel: Sized {
    fn for_monkeys(monkeys: &[Monkey]) -> Vec<Vec<Self>>;
    fn divisible_by(&self, divisor: &u64) -> bool;
    fn apply(&self, op: &Operation) -> Self;
}

impl WorryLevel for u64 {
    fn for_monkeys(monkeys: &[Monkey]) -> Vec<Vec<Self>> {
        monkeys.iter().map(|m| m.starting_items.clone()).collect()
    }

    fn divisible_by(&self, divisor: &u64) -> bool {
        self.is_multiple_of(*divisor)
    }

    fn apply(&self, op: &Operation) -> Self {
        match op {
            Operation::NoOp => *self,
            Operation::Add(x) => self + x,
            Operation::Mul(x) => self * x,
            Operation::MulBySelf => self * self,
            Operation::DivByThree => self / 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ModuloTableWorryLevel {
    modulos: HashMap<u64, u64>,
}

impl WorryLevel for ModuloTableWorryLevel {
    fn for_monkeys(monkeys: &[Monkey]) -> Vec<Vec<Self>> {
        let divisors: Vec<_> = monkeys.iter().map(|m| m.test_divisor).collect();
        monkeys
            .iter()
            .map(|m| {
                m.starting_items
                    .iter()
                    .map(|num| Self {
                        modulos: HashMap::from_iter(divisors.iter().map(|d| (*d, num % d))),
                    })
                    .collect()
            })
            .collect()
    }

    fn divisible_by(&self, divisor: &u64) -> bool {
        self.modulos.get(divisor).unwrap() == &0
    }

    fn apply(&self, op: &Operation) -> Self {
        match op {
            Operation::NoOp => self.clone(),
            Operation::Add(x) => Self {
                modulos: self
                    .modulos
                    .iter()
                    .map(|(div, modulo)| (*div, (modulo + x) % div))
                    .collect(),
            },
            Operation::Mul(x) => Self {
                modulos: self
                    .modulos
                    .iter()
                    .map(|(div, modulo)| (*div, (modulo * x) % div))
                    .collect(),
            },
            Operation::MulBySelf => Self {
                modulos: self
                    .modulos
                    .iter()
                    .map(|(div, modulo)| (*div, (modulo * modulo) % div))
                    .collect(),
            },
            Operation::DivByThree => panic!("Not implemented"),
        }
    }
}

#[derive(Debug)]
struct Game<'m, W: WorryLevel> {
    monkeys: &'m [Monkey],
    worry_levels: Vec<Vec<W>>,
    inspection_counts: Vec<usize>,
    end_of_round_op: Operation,
}

impl<'m, W: WorryLevel> Game<'m, W> {
    fn from(monkeys: &'m [Monkey], end_of_round_op: Operation) -> Self {
        Game {
            monkeys,
            worry_levels: W::for_monkeys(monkeys),
            inspection_counts: vec![0; monkeys.len()],
            end_of_round_op,
        }
    }

    fn play_round(&mut self) {
        for (idx, monkey) in self.monkeys.iter().enumerate() {
            let items = std::mem::take(&mut self.worry_levels[idx]);
            self.inspection_counts[idx] += items.len();
            for item in items {
                let new_worry = item.apply(&monkey.operation).apply(&self.end_of_round_op);
                let target = monkey.target(&new_worry);
                self.worry_levels[target].push(new_worry);
            }
        }
    }

    fn play_rounds(&mut self, number: usize) {
        for _ in 0..number {
            self.play_round();
        }
    }

    fn monkey_business(&self) -> usize {
        self.inspection_counts
            .iter()
            .cloned()
            .sorted()
            .rev()
            .take(2)
            .reduce(std::ops::Mul::mul)
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_is_parsed() {
        assert_eq!(parse_input(EXAMPLE_INPUT).unwrap(), example_monkeys());
    }

    #[test]
    fn worry_levels_after_first_round() {
        let monkeys = example_monkeys();
        let mut game = Game::<u64>::from(&monkeys, Operation::DivByThree);
        game.play_round();
        assert_eq!(game.worry_levels, WORRY_LEVELS_AFTER_FIRST_ROUND);
    }

    #[test]
    fn inspection_counts_after_20_rounds() {
        let monkeys = example_monkeys();
        let mut game = Game::<u64>::from(&monkeys, Operation::DivByThree);
        game.play_rounds(20);
        assert_eq!(game.inspection_counts, INSPECTION_COUNTS_AFTER_20_ROUNDS);
    }

    #[test]
    fn answer_for_part_1() {
        let monkeys = example_monkeys();
        let mut game = Game::<u64>::from(&monkeys, Operation::DivByThree);
        game.play_rounds(20);
        assert_eq!(game.monkey_business(), 10605);
    }

    #[test]
    fn answer_for_part_2() {
        let monkeys = example_monkeys();
        let mut game = Game::<ModuloTableWorryLevel>::from(&monkeys, Operation::NoOp);
        game.play_rounds(10000);
        assert_eq!(game.monkey_business(), 2713310158);
    }

    const WORRY_LEVELS_AFTER_FIRST_ROUND: &[&[u64]] = &[
        &[20, 23, 27, 26],
        &[2080, 25, 167, 207, 401, 1046],
        &[],
        &[],
    ];

    const INSPECTION_COUNTS_AFTER_20_ROUNDS: &[usize] = &[101, 95, 7, 105];

    const EXAMPLE_INPUT: &str = "\
Monkey 0:
  Starting items: 79, 98
  Operation: new = old * 19
  Test: divisible by 23
    If true: throw to monkey 2
    If false: throw to monkey 3

Monkey 1:
  Starting items: 54, 65, 75, 74
  Operation: new = old + 6
  Test: divisible by 19
    If true: throw to monkey 2
    If false: throw to monkey 0

Monkey 2:
  Starting items: 79, 60, 97
  Operation: new = old * old
  Test: divisible by 13
    If true: throw to monkey 1
    If false: throw to monkey 3

Monkey 3:
  Starting items: 74
  Operation: new = old + 3
  Test: divisible by 17
    If true: throw to monkey 0
    If false: throw to monkey 1";

    fn example_monkeys() -> [Monkey; 4] {
        [
            Monkey {
                starting_items: vec![79, 98],
                operation: Operation::Mul(19),
                test_divisor: 23,
                true_target: 2,
                false_target: 3,
            },
            Monkey {
                starting_items: vec![54, 65, 75, 74],
                operation: Operation::Add(6),
                test_divisor: 19,
                true_target: 2,
                false_target: 0,
            },
            Monkey {
                starting_items: vec![79, 60, 97],
                operation: Operation::MulBySelf,
                test_divisor: 13,
                true_target: 1,
                false_target: 3,
            },
            Monkey {
                starting_items: vec![74],
                operation: Operation::Add(3),
                test_divisor: 17,
                true_target: 0,
                false_target: 1,
            },
        ]
    }
}
