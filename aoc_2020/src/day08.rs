use anyhow::{anyhow, bail};
use aoc_companion::prelude::*;
use itertools::Itertools;

use std::collections::HashSet;

pub(crate) struct Door {
    program: Vec<Instruction>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        use Instruction::*;
        input
            .lines()
            .map(|line| line.split_once(' ').unwrap())
            .map(|(op, arg)| {
                Ok(match op {
                    "nop" => Nop(arg.parse()?),
                    "acc" => Acc(arg.parse()?),
                    "jmp" => Jmp(arg.parse()?),
                    _ => bail!("invalid instruction: {op:?}"),
                })
            })
            .try_collect()
            .map(|program| Door { program })
    }

    fn part1(&self) -> Result<i32> {
        use ProgramOutcome::*;
        match run_program(&self.program) {
            Loops(acc) => Ok(acc),
            Terminates(_) => bail!("program is expected to loop, but terminated"),
        }
    }

    fn part2(&self) -> Result<i32> {
        find_terminating_program(&self.program).ok_or_else(|| {
            anyhow!("could not find a terminating program by switching jmp <-> nop instructions")
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Instruction {
    Nop(i32),
    Acc(i32),
    Jmp(i32),
}

impl Instruction {
    fn switched(&self) -> Self {
        use Instruction::*;
        match *self {
            Nop(i) => Jmp(i),
            Acc(i) => Acc(i),
            Jmp(i) => Nop(i),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ProgramOutcome {
    Terminates(i32),
    Loops(i32),
}

fn run_program(instructions: &[Instruction]) -> ProgramOutcome {
    use Instruction::*;
    let mut used_instructions = HashSet::<usize>::new();
    let mut current_index = 0;
    let mut acc = 0;

    while !used_instructions.contains(&current_index) {
        used_instructions.insert(current_index);

        if let Some(current_instruction) = instructions.get(current_index) {
            if let Acc(i) = current_instruction {
                acc += i;
            }

            match *current_instruction {
                Nop(_) | Acc(_) => current_index += 1,
                Jmp(i) if i < 0 => current_index -= (-i) as usize,
                Jmp(i) => current_index += i as usize,
            };
        } else {
            return ProgramOutcome::Terminates(acc);
        }
    }

    ProgramOutcome::Loops(acc)
}

fn modified_program(original_program: &[Instruction], index: usize) -> Vec<Instruction> {
    original_program
        .iter()
        .enumerate()
        .map(|(j, instr)| if j == index { instr.switched() } else { *instr })
        .collect()
}

fn find_terminating_program(program: &[Instruction]) -> Option<i32> {
    (0..program.len())
        .map(|j| modified_program(program, j))
        .map(|program| run_program(&program))
        .find_map(|outcome| match outcome {
            ProgramOutcome::Terminates(result) => Some(result),
            ProgramOutcome::Loops(_) => None,
        })
}

#[cfg(test)]
mod test {
    use super::Instruction::*;
    use super::*;

    const LOOPING_EXAMPLE_INPUT: &str = "\
nop +0
acc +1
jmp +4
acc +3
jmp -3
acc -99
acc +1
jmp -4
acc +6";

    const LOOPING_EXAMPLE_PROGRAM: &[Instruction] = &[
        Nop(0),
        Acc(1),
        Jmp(4),
        Acc(3),
        Jmp(-3),
        Acc(-99),
        Acc(1),
        Jmp(-4),
        Acc(6),
    ];

    const TERMINATING_EXAMPLE_INPUT: &str = "\
nop +0
acc +1
jmp +4
acc +3
jmp -3
acc -99
acc +1
nop -4
acc +6";

    const TERMINATING_EXAMPLE_PROGRAM: &[Instruction] = &[
        Nop(0),
        Acc(1),
        Jmp(4),
        Acc(3),
        Jmp(-3),
        Acc(-99),
        Acc(1),
        Nop(-4),
        Acc(6),
    ];

    #[test]
    fn parse_program() {
        assert_eq!(
            Door::parse(LOOPING_EXAMPLE_INPUT).unwrap().program,
            LOOPING_EXAMPLE_PROGRAM,
        );
        assert_eq!(
            Door::parse(TERMINATING_EXAMPLE_INPUT).unwrap().program,
            TERMINATING_EXAMPLE_PROGRAM,
        );
    }

    #[test]
    fn program_loops() {
        assert_eq!(
            run_program(LOOPING_EXAMPLE_PROGRAM),
            ProgramOutcome::Loops(5)
        );
    }

    #[test]
    fn program_terminates() {
        assert_eq!(
            run_program(TERMINATING_EXAMPLE_PROGRAM),
            ProgramOutcome::Terminates(8)
        );
    }

    #[test]
    fn terminating_program_found() {
        assert_eq!(find_terminating_program(LOOPING_EXAMPLE_PROGRAM), Some(8));
    }
}
