use aoc_companion::prelude::*;

use itertools::Itertools;
use thiserror::Error;

use std::collections::HashMap;
use std::num::ParseIntError;
use std::str::FromStr;

pub struct Door {
    monkeys: HashMap<String, Monkey>,
}

impl ParseInput<'_> for Door {
    type Error = ParseError;

    fn parse(input: &str) -> Result<Self, Self::Error> {
        parse_input(input).map(|monkeys| Self { monkeys })
    }
}

impl Part1 for Door {
    type Output = isize;
    type Error = RuntimeError;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        solve_for("root", &self.monkeys)
    }
}

impl Part2 for Door {
    type Output = isize;
    type Error = RuntimeError;

    fn part2(&self) -> Result<Self::Output, Self::Error> {
        let transformed = transform_for_part2(&self.monkeys)?;
        solve_for("humn", &transformed)
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Assignment not found on line: {0}")]
    AssignmentNotFound(String),
    #[error("Operator expression not found: {0}")]
    OperatorExpressionNotFound(String),
    #[error("Unrecognized operator: {0}")]
    UnrecognizedOperator(String),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Could not find an equation involving {0:?}")]
    CouldNotFindAnEquationInvolvingVariable(String),
    #[error("Operation cannot be inverted")]
    NoInversion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Monkey {
    Constant(isize),
    Operation {
        lhs: String,
        rhs: String,
        ops: fn(isize, isize) -> isize,
    },
}

enum Resolution {
    Ready(isize),
    Dependent(Vec<String>),
}

impl Monkey {
    fn resolve(&self, data: &HashMap<String, isize>) -> Resolution {
        match self {
            Monkey::Constant(num) => Resolution::Ready(*num),
            Monkey::Operation { lhs, rhs, ops } => match (data.get(lhs), data.get(rhs)) {
                (Some(_), None) => Resolution::Dependent(vec![rhs.clone()]),
                (None, Some(_)) => Resolution::Dependent(vec![lhs.clone()]),
                (None, None) => Resolution::Dependent(vec![lhs.clone(), rhs.clone()]),
                (Some(&lhs), Some(&rhs)) => Resolution::Ready(ops(lhs, rhs)),
            },
        }
    }
}

impl FromStr for Monkey {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains(' ') {
            let (lhs_str, rest) = s.split_once(' ').unwrap();
            let (ops_str, rhs_str) = rest
                .split_once(' ')
                .ok_or_else(|| ParseError::OperatorExpressionNotFound(rest.to_string()))?;
            let ops = match ops_str {
                "+" => Ok(std::ops::Add::add as fn(isize, isize) -> isize),
                "-" => Ok(std::ops::Sub::sub as fn(isize, isize) -> isize),
                "*" => Ok(std::ops::Mul::mul as fn(isize, isize) -> isize),
                "/" => Ok(std::ops::Div::div as fn(isize, isize) -> isize),
                _ => Err(ParseError::UnrecognizedOperator(ops_str.to_owned())),
            }?;
            Ok(Monkey::Operation {
                lhs: lhs_str.to_owned(),
                rhs: rhs_str.to_owned(),
                ops,
            })
        } else {
            Ok(Monkey::Constant(s.parse()?))
        }
    }
}

fn parse_input(input: &str) -> Result<HashMap<String, Monkey>, ParseError> {
    input
        .lines()
        .map(|line| {
            let (name, expr) = line
                .split_once(": ")
                .ok_or_else(|| ParseError::AssignmentNotFound(line.to_owned()))?;
            Ok((name.to_owned(), expr.parse()?))
        })
        .try_collect()
}

fn solve_for(target: &str, monkeys: &HashMap<String, Monkey>) -> Result<isize, RuntimeError> {
    let mut data = HashMap::new();
    let mut dependencies = vec![target.to_owned()];
    while let Some(dependency) = dependencies.pop() {
        match monkeys
            .get(&dependency)
            .ok_or_else(|| {
                RuntimeError::CouldNotFindAnEquationInvolvingVariable(dependency.clone())
            })?
            .resolve(&data)
        {
            Resolution::Ready(num) => {
                data.insert(dependency.to_owned(), num);
            }
            Resolution::Dependent(subdependencies) => {
                dependencies.push(dependency);
                dependencies.extend(subdependencies);
            }
        }
    }

    Ok(data[target])
}

// t = x + a => x = t - a
// t = x - a => x = t + a
// t = x * a => x = t / a
// t = x / a => x = t * a
fn inv_lhs(f: fn(isize, isize) -> isize) -> Result<fn(isize, isize) -> isize, RuntimeError> {
    match f {
        f if f == std::ops::Add::add => Ok(std::ops::Sub::sub),
        f if f == std::ops::Sub::sub => Ok(std::ops::Add::add),
        f if f == std::ops::Mul::mul => Ok(std::ops::Div::div),
        f if f == std::ops::Div::div => Ok(std::ops::Mul::mul),
        _ => Err(RuntimeError::NoInversion),
    }
}

// t = a + x => x = t - a
// t = a - x => x = a - t
// t = a * x => x = t / a
// t = a / x => x = a / t
fn inv_rhs(f: fn(isize, isize) -> isize) -> Result<fn(isize, isize) -> isize, RuntimeError> {
    match f {
        f if f == std::ops::Add::add => Ok(std::ops::Sub::sub),
        f if f == std::ops::Sub::sub => Ok(|t, a| a - t),
        f if f == std::ops::Mul::mul => Ok(std::ops::Div::div),
        f if f == std::ops::Div::div => Ok(|t, a| a / t),
        _ => Err(RuntimeError::NoInversion),
    }
}

fn transform_for_part2(
    monkeys: &HashMap<String, Monkey>,
) -> Result<HashMap<String, Monkey>, RuntimeError> {
    let mut current = "humn".to_string();
    let mut new_monkeys = monkeys.clone();
    let other = loop {
        let (name, other, ops) = monkeys
            .iter()
            .find_map(|(name, monkey)| match monkey {
                Monkey::Operation { lhs, rhs, ops } if lhs == &current => {
                    Some((name, rhs, inv_lhs(*ops)))
                }
                Monkey::Operation { lhs, rhs, ops } if rhs == &current => {
                    Some((name, lhs, inv_rhs(*ops)))
                }
                _ => None,
            })
            .ok_or_else(|| {
                RuntimeError::CouldNotFindAnEquationInvolvingVariable(current.clone())
            })?;

        let new = Monkey::Operation {
            lhs: name.clone(),
            rhs: other.clone(),
            ops: ops?,
        };
        let next = name.clone();
        new_monkeys.remove(&next);

        if next == "root" {
            break other;
        }

        new_monkeys.insert(current, new);
        current = next;
    };

    if let Some(name) = new_monkeys.values_mut().find_map(|monkey| match monkey {
        Monkey::Operation { lhs, .. } if lhs == &current => Some(lhs),
        Monkey::Operation { rhs, .. } if rhs == &current => Some(rhs),
        _ => None,
    }) {
        *name = other.clone();
        Ok(new_monkeys)
    } else {
        Err(RuntimeError::CouldNotFindAnEquationInvolvingVariable(
            current,
        ))
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
    fn root_can_be_solved_for() {
        assert_eq!(solve_for("root", &example_monkeys()).unwrap(), 152);
    }

    #[test]
    fn transformed_equations_can_be_solved_for_humn() {
        assert_eq!(
            solve_for("humn", &parse_input(TRANSFORMED_EXAMPLE).unwrap()).unwrap(),
            301
        );
    }

    #[test]
    fn transform_example_for_part2() {
        assert_eq!(
            transform_for_part2(&example_monkeys()).unwrap(),
            parse_input(TRANSFORMED_EXAMPLE).unwrap()
        );
    }

    const EXAMPLE_INPUT: &str = "\
root: pppw + sjmn
dbpl: 5
cczh: sllz + lgvd
zczc: 2
ptdq: humn - dvpt
dvpt: 3
lfqf: 4
humn: 5
ljgn: 2
sjmn: drzm * dbpl
sllz: 4
pppw: cczh / lfqf
lgvd: ljgn * ptdq
drzm: hmdt - zczc
hmdt: 32";

    const TRANSFORMED_EXAMPLE: &str = "\
dbpl: 5
zczc: 2
dvpt: 3
lfqf: 4
humn: 5
ljgn: 2
sjmn: drzm * dbpl
sllz: 4
drzm: hmdt - zczc
hmdt: 32
humn: ptdq + dvpt
ptdq: lgvd / ljgn
lgvd: cczh - sllz
cczh: sjmn * lfqf";

    fn example_monkeys() -> HashMap<String, Monkey> {
        HashMap::from([
            (
                "root".to_owned(),
                Monkey::Operation {
                    lhs: "pppw".to_owned(),
                    rhs: "sjmn".to_owned(),
                    ops: std::ops::Add::add,
                },
            ),
            ("dbpl".to_owned(), Monkey::Constant(5)),
            (
                "cczh".to_owned(),
                Monkey::Operation {
                    lhs: "sllz".to_owned(),
                    rhs: "lgvd".to_owned(),
                    ops: std::ops::Add::add,
                },
            ),
            ("zczc".to_owned(), Monkey::Constant(2)),
            (
                "ptdq".to_owned(),
                Monkey::Operation {
                    lhs: "humn".to_owned(),
                    rhs: "dvpt".to_owned(),
                    ops: std::ops::Sub::sub,
                },
            ),
            ("dvpt".to_owned(), Monkey::Constant(3)),
            ("lfqf".to_owned(), Monkey::Constant(4)),
            ("humn".to_owned(), Monkey::Constant(5)),
            ("ljgn".to_owned(), Monkey::Constant(2)),
            (
                "sjmn".to_owned(),
                Monkey::Operation {
                    lhs: "drzm".to_owned(),
                    rhs: "dbpl".to_owned(),
                    ops: std::ops::Mul::mul,
                },
            ),
            ("sllz".to_owned(), Monkey::Constant(4)),
            (
                "pppw".to_owned(),
                Monkey::Operation {
                    lhs: "cczh".to_owned(),
                    rhs: "lfqf".to_owned(),
                    ops: std::ops::Div::div,
                },
            ),
            (
                "lgvd".to_owned(),
                Monkey::Operation {
                    lhs: "ljgn".to_owned(),
                    rhs: "ptdq".to_owned(),
                    ops: std::ops::Mul::mul,
                },
            ),
            (
                "drzm".to_owned(),
                Monkey::Operation {
                    lhs: "hmdt".to_owned(),
                    rhs: "zczc".to_owned(),
                    ops: std::ops::Sub::sub,
                },
            ),
            ("hmdt".to_owned(), Monkey::Constant(32)),
        ])
    }
}
