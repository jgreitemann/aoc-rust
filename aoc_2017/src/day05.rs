use aoc_companion::prelude::*;

use JumpChangePolicy::*;

#[derive(Debug, Eq, PartialEq)]
enum JumpChangePolicy {
    AlwaysIncrease,
    DecreaseLongJumps,
}

pub struct Door {
    jumps: Vec<isize>,
}

impl ParseInput for Door {
    type Error = std::num::ParseIntError;

    fn parse(input: &str) -> Result<Self, Self::Error> {
        input
            .lines()
            .map(|s| s.trim().parse())
            .collect::<Result<Vec<_>, _>>()
            .map(|jumps| Self { jumps })
    }
}

impl Part1 for Door {
    type Output = usize;
    type Error = std::convert::Infallible;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        Ok(Program::new(self.jumps.clone(), AlwaysIncrease).count())
    }
}

impl Part2 for Door {
    type Output = usize;
    type Error = std::convert::Infallible;

    fn part2(&self) -> Result<Self::Output, Self::Error> {
        Ok(Program::new(self.jumps.clone(), DecreaseLongJumps).count())
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Program {
    pc: usize,
    prog: Vec<isize>,
    policy: JumpChangePolicy,
}

impl Program {
    fn new(prog: Vec<isize>, policy: JumpChangePolicy) -> Self {
        Self {
            pc: 0,
            prog,
            policy,
        }
    }
}

impl Iterator for Program {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if (0..self.prog.len()).contains(&self.pc) {
            let jump = &mut self.prog[self.pc];
            let pc = self.pc;
            Some(std::mem::replace(
                &mut self.pc,
                match std::mem::replace(
                    jump,
                    match self.policy {
                        DecreaseLongJumps if *jump >= 3 => *jump - 1,
                        _ => *jump + 1,
                    },
                ) {
                    d if d >= 0 => pc.checked_add(d as usize).unwrap(),
                    d => pc.wrapping_sub(-d as usize),
                },
            ))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use itertools::assert_equal;

    use super::*;

    #[test]
    fn program_execution_produces_intermediate_program_counters() {
        assert_equal(
            Program::new(vec![0, 3, 0, 1, -3], AlwaysIncrease),
            [0, 0, 1, 4, 1],
        );
    }

    #[test]
    fn program_terminates_after_correct_number_of_jumps() {
        assert_eq!(
            Program::new(vec![0, 3, 0, 1, -3], AlwaysIncrease).count(),
            5
        );
        assert_eq!(
            Program::new(vec![0, 3, 0, 1, -3], DecreaseLongJumps).count(),
            10
        );
    }
}
