use std::sync::OnceLock;

use aoc_companion::prelude::*;
use regex::Regex;

pub(crate) struct Door<'input> {
    input: &'input str,
}

impl<'input> Solution<'input> for Door<'input> {
    fn parse(input: &'input str) -> Door<'input> {
        Door { input }
    }

    fn part1(&self) -> u32 {
        execute_instructions(op_iter(self.input).filter(|op| matches!(op, Op::Mul(_, _))))
    }

    fn part2(&self) -> u32 {
        execute_instructions(op_iter(self.input))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Mul(u32, u32),
    Do,
    Dont,
}

fn op_iter(input: &str) -> impl Iterator<Item = Op> + use<'_> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r"(?:mul\(([0-9]{1,3}),([0-9]{1,3})\)|do\(\)|don't\(\))").unwrap()
    });
    re.captures_iter(input)
        .map(|c| match c.get(0).unwrap().as_str() {
            "do()" => Op::Do,
            "don't()" => Op::Dont,
            _ => Op::Mul(
                c.get(1).unwrap().as_str().parse().unwrap(),
                c.get(2).unwrap().as_str().parse().unwrap(),
            ),
        })
}

fn execute_instructions(op_iter: impl Iterator<Item = Op>) -> u32 {
    op_iter
        .scan(true, |enabled, op| {
            Some(match op {
                Op::Mul(lhs, rhs) => enabled.then(|| lhs * rhs),
                Op::Do => {
                    *enabled = true;
                    None
                }
                Op::Dont => {
                    *enabled = false;
                    None
                }
            })
        })
        .flatten()
        .sum()
}

#[cfg(test)]
mod tests {
    use itertools::assert_equal;

    use super::*;

    const EXAMPLE_INPUT_PART1: &str =
        "xmul(2,4)%&mul[3,7]!@^do_not_mul(5,5)+mul(32,64]then(mul(11,8)mul(8,5))";
    const EXAMPLE_OPS_PART1: &[Op] = &[Op::Mul(2, 4), Op::Mul(5, 5), Op::Mul(11, 8), Op::Mul(8, 5)];

    const EXAMPLE_INPUT_PART2: &str =
        "xmul(2,4)&mul[3,7]!^don't()_mul(5,5)+mul(32,64](mul(11,8)undo()?mul(8,5))";
    const EXAMPLE_OPS_PART2: &[Op] = &[
        Op::Mul(2, 4),
        Op::Dont,
        Op::Mul(5, 5),
        Op::Mul(11, 8),
        Op::Do,
        Op::Mul(8, 5),
    ];

    #[test]
    fn mul_instructions_are_found() {
        assert_equal(
            op_iter(EXAMPLE_INPUT_PART1),
            EXAMPLE_OPS_PART1.iter().copied(),
        );
    }

    #[test]
    fn all_instructions_are_found() {
        assert_equal(
            op_iter(EXAMPLE_INPUT_PART2),
            EXAMPLE_OPS_PART2.iter().copied(),
        );
    }

    #[test]
    fn sum_of_muls_for_example() {
        assert_eq!(execute_instructions(EXAMPLE_OPS_PART1.iter().copied()), 161);
    }

    #[test]
    fn do_and_dont_are_considered_for_example() {
        assert_eq!(execute_instructions(EXAMPLE_OPS_PART2.iter().copied()), 48);
    }
}
