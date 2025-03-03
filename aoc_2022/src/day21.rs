use aoc_companion::prelude::*;

use itertools::Itertools;
use thiserror::Error;

use std::collections::HashMap;
use std::num::ParseIntError;
use std::str::FromStr;

pub(crate) struct Door {
    monkeys: HashMap<String, Monkey>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseError> {
        parse_input(input).map(|monkeys| Self { monkeys })
    }

    fn part1(&self) -> Result<isize, RuntimeError> {
        solve_for("root", &self.monkeys)
    }

    fn part2(&self) -> Result<isize, RuntimeError> {
        let transformed = transform_for_part2(&self.monkeys)?;
        solve_for("humn", &transformed)
    }
}

#[derive(Debug, Error)]
pub(crate) enum ParseError {
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
pub(crate) enum RuntimeError {
    #[error("Could not find an equation involving {0:?}")]
    CouldNotFindAnEquationInvolvingVariable(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    SubInv,
    DivInv,
}

impl Operator {
    fn apply(&self, lhs: isize, rhs: isize) -> isize {
        match self {
            Operator::Add => lhs + rhs,
            Operator::Sub => lhs - rhs,
            Operator::Mul => lhs * rhs,
            Operator::Div => lhs / rhs,
            Operator::SubInv => rhs - lhs,
            Operator::DivInv => rhs / lhs,
        }
    }

    fn inv_lhs(&self) -> Self {
        // t = x + a => x = t - a
        // t = x - a => x = t + a
        // t = x * a => x = t / a
        // t = x / a => x = t * a
        // t = a - x => x = a - t
        // t = a / x => x = a / t
        use Operator::*;
        match *self {
            Add => Sub,
            Sub => Add,
            Mul => Div,
            Div => Mul,
            SubInv => SubInv,
            DivInv => DivInv,
        }
    }

    fn inv_rhs(&self) -> Self {
        // t = a + x => x = t - a
        // t = a - x => x = a - t
        // t = a * x => x = t / a
        // t = a / x => x = a / t
        // t = x - a => x = t + a
        // t = x / a => x = t * a
        use Operator::*;
        match *self {
            Add => Sub,
            Sub => SubInv,
            Mul => Div,
            Div => DivInv,
            SubInv => Add,
            DivInv => Mul,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Monkey {
    Constant(isize),
    Operation {
        lhs: String,
        rhs: String,
        ops: Operator,
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
                (Some(&lhs), Some(&rhs)) => Resolution::Ready(ops.apply(lhs, rhs)),
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
                "+" => Ok(Operator::Add),
                "-" => Ok(Operator::Sub),
                "*" => Ok(Operator::Mul),
                "/" => Ok(Operator::Div),
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
                    Some((name, rhs, ops.inv_lhs()))
                }
                Monkey::Operation { lhs, rhs, ops } if rhs == &current => {
                    Some((name, lhs, ops.inv_rhs()))
                }
                _ => None,
            })
            .ok_or_else(|| {
                RuntimeError::CouldNotFindAnEquationInvolvingVariable(current.clone())
            })?;

        let new = Monkey::Operation {
            lhs: name.clone(),
            rhs: other.clone(),
            ops,
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
                    ops: Operator::Add,
                },
            ),
            ("dbpl".to_owned(), Monkey::Constant(5)),
            (
                "cczh".to_owned(),
                Monkey::Operation {
                    lhs: "sllz".to_owned(),
                    rhs: "lgvd".to_owned(),
                    ops: Operator::Add,
                },
            ),
            ("zczc".to_owned(), Monkey::Constant(2)),
            (
                "ptdq".to_owned(),
                Monkey::Operation {
                    lhs: "humn".to_owned(),
                    rhs: "dvpt".to_owned(),
                    ops: Operator::Sub,
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
                    ops: Operator::Mul,
                },
            ),
            ("sllz".to_owned(), Monkey::Constant(4)),
            (
                "pppw".to_owned(),
                Monkey::Operation {
                    lhs: "cczh".to_owned(),
                    rhs: "lfqf".to_owned(),
                    ops: Operator::Div,
                },
            ),
            (
                "lgvd".to_owned(),
                Monkey::Operation {
                    lhs: "ljgn".to_owned(),
                    rhs: "ptdq".to_owned(),
                    ops: Operator::Mul,
                },
            ),
            (
                "drzm".to_owned(),
                Monkey::Operation {
                    lhs: "hmdt".to_owned(),
                    rhs: "zczc".to_owned(),
                    ops: Operator::Sub,
                },
            ),
            ("hmdt".to_owned(), Monkey::Constant(32)),
        ])
    }
}
