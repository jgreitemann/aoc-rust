use anyhow::bail;
use aoc_companion::prelude::*;
use aoc_utils::linalg::Vector;
use itertools::Itertools;

const OFFSET: i64 = 10000000000000;
const COST: Vector<i64, 2> = Vector([3, 1]);

pub(crate) struct Door {
    machines: Vec<Machine>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Door> {
        let machines = input
            .split("\n\n")
            .map(|paragraph| paragraph.parse())
            .try_collect()?;
        Ok(Door { machines })
    }

    fn part1(&self) -> i64 {
        tokens_needed(&self.machines)
    }

    fn part2(&self) -> impl door::IntoResult {
        let adjusted = self
            .machines
            .iter()
            .map(Machine::adjust_prize)
            .collect_vec();
        tokens_needed(&adjusted)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Machine {
    a: Vector<i64, 2>,
    b: Vector<i64, 2>,
    prize: Vector<i64, 2>,
}

impl std::str::FromStr for Machine {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut lines = s.trim().lines();
        let Some(a_line) = lines.next() else {
            bail!("missing line describing button A");
        };
        let Some(b_line) = lines.next() else {
            bail!("missing line describing button B");
        };
        let Some(prize_line) = lines.next() else {
            bail!("missing line describing prize");
        };
        let Some(a_line) = a_line.strip_prefix("Button A: ") else {
            bail!("missing prefix for button A");
        };
        let Some(b_line) = b_line.strip_prefix("Button B: ") else {
            bail!("missing prefix for button B");
        };
        let Some(prize_line) = prize_line.strip_prefix("Prize: ") else {
            bail!("missing prefix for prize");
        };

        let parse_vector = |str: &str, introducer: char| -> Result<Vector<i64, 2>> {
            let Some((lhs, rhs)) = str.split_once(", ") else {
                bail!("missing comma delimiting coordinates");
            };
            let Some(lhs) = lhs.strip_prefix('X') else {
                bail!("missing coordinate name 'X'");
            };
            let Some(rhs) = rhs.strip_prefix('Y') else {
                bail!("missing coordinate name 'Y'");
            };
            let Some(lhs) = lhs.strip_prefix(introducer) else {
                bail!("missing coordinate introducer {introducer:?}");
            };
            let Some(rhs) = rhs.strip_prefix(introducer) else {
                bail!("missing coordinate introducer {introducer:?}");
            };
            Ok(Vector([lhs.parse()?, rhs.parse()?]))
        };

        Ok(Machine {
            a: parse_vector(a_line, '+')?,
            b: parse_vector(b_line, '+')?,
            prize: parse_vector(prize_line, '=')?,
        })
    }
}

impl Machine {
    fn solve(&self) -> Option<Vector<i64, 2>> {
        let discriminant = self.a[0] * self.b[1] - self.a[1] * self.b[0];
        assert_ne!(discriminant, 0);
        let cramer_a = self.prize[0] * self.b[1] - self.prize[1] * self.b[0];
        let cramer_b = self.a[0] * self.prize[1] - self.a[1] * self.prize[0];
        if cramer_a % discriminant != 0 || cramer_b % discriminant != 0 {
            return None;
        }
        Some(Vector([cramer_a / discriminant, cramer_b / discriminant]))
    }

    fn adjust_prize(&self) -> Self {
        Machine {
            a: self.a,
            b: self.b,
            prize: self.prize + Vector([OFFSET, OFFSET]),
        }
    }
}

fn token_cost(solution: Vector<i64, 2>) -> i64 {
    (solution * COST).norm_l1()
}

fn tokens_needed(machines: &[Machine]) -> i64 {
    machines
        .iter()
        .filter_map(|m| m.solve())
        .map(token_cost)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "\
Button A: X+94, Y+34
Button B: X+22, Y+67
Prize: X=8400, Y=5400

Button A: X+26, Y+66
Button B: X+67, Y+21
Prize: X=12748, Y=12176

Button A: X+17, Y+86
Button B: X+84, Y+37
Prize: X=7870, Y=6450

Button A: X+69, Y+23
Button B: X+27, Y+71
Prize: X=18641, Y=10279";

    const EXAMPLE_MACHINES: [Machine; 4] = [
        Machine {
            a: Vector([94, 34]),
            b: Vector([22, 67]),
            prize: Vector([8400, 5400]),
        },
        Machine {
            a: Vector([26, 66]),
            b: Vector([67, 21]),
            prize: Vector([12748, 12176]),
        },
        Machine {
            a: Vector([17, 86]),
            b: Vector([84, 37]),
            prize: Vector([7870, 6450]),
        },
        Machine {
            a: Vector([69, 23]),
            b: Vector([27, 71]),
            prize: Vector([18641, 10279]),
        },
    ];

    #[test]
    fn parse_example_input() {
        let door = Door::parse(EXAMPLE_INPUT).unwrap();
        assert_eq!(door.machines, EXAMPLE_MACHINES);
    }

    #[test]
    fn tokens_needed_in_example() {
        assert_eq!(tokens_needed(&EXAMPLE_MACHINES), 480);
    }

    #[test]
    fn tokens_needed_with_adjusted_prize() {
        assert_eq!(
            tokens_needed(&EXAMPLE_MACHINES.map(|m| m.adjust_prize())),
            875318608908
        );
    }
}
