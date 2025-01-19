use anyhow::Context;
use aoc_companion::prelude::*;
use itertools::Itertools;
use rayon::prelude::*;

pub(crate) struct Door(Vec<Equation>);

#[derive(Debug, Clone, PartialEq, Eq)]
struct Equation {
    test_value: i64,
    operands: Vec<i64>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        parse_equations(input).map(Door)
    }

    fn part1(&self) -> i64 {
        total_test_value_of_equations_fulfilled_by(&ADD_AND_MUL, &self.0)
    }

    fn part2(&self) -> i64 {
        total_test_value_of_equations_fulfilled_by(&ALL_OPS, &self.0)
    }
}

const SEP: &str = ": ";

fn parse_equations(input: &str) -> Result<Vec<Equation>> {
    input
        .lines()
        .map(|line| {
            let Some((res_str, ops_str)) = line.split_once(SEP) else {
                anyhow::bail!("missing {SEP:?} separator in equation line");
            };
            Ok(Equation {
                test_value: res_str
                    .parse()
                    .with_context(|| format!("failed to parse equation test value {res_str:?}"))?,
                operands: ops_str
                    .split(' ')
                    .map(|op| {
                        op.parse()
                            .with_context(|| format!("failed to parse equation operand {op:?}"))
                    })
                    .try_collect()?,
            })
        })
        .try_collect()
}

enum Op {
    Add,
    Mul,
    Concat,
}

const ADD_AND_MUL: [Op; 2] = [Op::Add, Op::Mul];
const ALL_OPS: [Op; 3] = [Op::Add, Op::Mul, Op::Concat];

impl Equation {
    fn can_be_fulfilled_with(&self, ops: &[Op]) -> bool {
        std::iter::repeat_n(ops, self.operands.len() - 1)
            .multi_cartesian_product()
            .any(|op_seq| {
                std::iter::once(&Op::Add)
                    .chain(op_seq)
                    .zip(self.operands.iter())
                    .fold(0, |acc, (op, num)| match op {
                        Op::Add => acc + num,
                        Op::Mul => acc * num,
                        Op::Concat => 10i64.pow(num.ilog10() + 1) * acc + num,
                    })
                    == self.test_value
            })
    }
}

fn total_test_value_of_equations_fulfilled_by(ops: &[Op], eqns: &[Equation]) -> i64 {
    eqns.par_iter()
        .filter(|eqn| eqn.can_be_fulfilled_with(ops))
        .map(|eqn| eqn.test_value)
        .sum()
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use super::*;

    const EXAMPLE_INPUT: &str = "\
190: 10 19
3267: 81 40 27
83: 17 5
156: 15 6
7290: 6 8 6 15
161011: 16 10 13
192: 17 8 14
21037: 9 7 18 13
292: 11 6 16 20";

    static EXAMPLE_EQNS: LazyLock<[Equation; 9]> = LazyLock::new(|| {
        [
            Equation {
                test_value: 190,
                operands: vec![10, 19],
            },
            Equation {
                test_value: 3267,
                operands: vec![81, 40, 27],
            },
            Equation {
                test_value: 83,
                operands: vec![17, 5],
            },
            Equation {
                test_value: 156,
                operands: vec![15, 6],
            },
            Equation {
                test_value: 7290,
                operands: vec![6, 8, 6, 15],
            },
            Equation {
                test_value: 161011,
                operands: vec![16, 10, 13],
            },
            Equation {
                test_value: 192,
                operands: vec![17, 8, 14],
            },
            Equation {
                test_value: 21037,
                operands: vec![9, 7, 18, 13],
            },
            Equation {
                test_value: 292,
                operands: vec![11, 6, 16, 20],
            },
        ]
    });

    #[test]
    fn parse_example_input() {
        assert_eq!(parse_equations(EXAMPLE_INPUT).unwrap(), *EXAMPLE_EQNS);
    }

    #[test]
    fn example_equations_fulfilled_by_combination_of_add_and_mul() {
        assert_eq!(
            EXAMPLE_EQNS
                .each_ref()
                .map(|eqn| eqn.can_be_fulfilled_with(&ADD_AND_MUL)),
            [true, true, false, false, false, false, false, false, true]
        );
    }

    #[test]
    fn example_equations_fulfilled_by_combination_of_all_operators() {
        assert_eq!(
            EXAMPLE_EQNS
                .each_ref()
                .map(|eqn| eqn.can_be_fulfilled_with(&ALL_OPS)),
            [true, true, false, true, true, false, true, false, true]
        );
    }

    #[test]
    fn total_test_values_of_equations_fulfilled_by_combination_of_add_and_mul() {
        assert_eq!(
            total_test_value_of_equations_fulfilled_by(&ADD_AND_MUL, EXAMPLE_EQNS.as_slice()),
            3749
        );
    }

    #[test]
    fn total_test_values_of_equations_fulfilled_by_combination_of_all_operators() {
        assert_eq!(
            total_test_value_of_equations_fulfilled_by(&ALL_OPS, EXAMPLE_EQNS.as_slice()),
            11387
        );
    }
}
