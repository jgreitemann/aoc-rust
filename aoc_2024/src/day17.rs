use anyhow::{anyhow, bail};
use aoc_companion::prelude::*;
use aoc_utils::array;
use itertools::{Itertools, iterate};

pub(crate) struct Door {
    initial_registers: [u64; 3],
    program: Vec<u8>,
}

struct Computer<'p> {
    ip: usize,
    registers: [u64; 3],
    program: &'p [u8],
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        let Some((register_str, program_str)) = input.split_once("\n\n") else {
            bail!("Missing empty line separating registers from program");
        };
        let initial_registers =
            array::try_from_iter(register_str.lines().map(|line| -> Result<u64> {
                Ok(line
                    .strip_prefix("Register ")
                    .ok_or_else(|| anyhow!("Register prefix introducer missing"))?
                    .strip_prefix(|c: char| c.is_ascii_uppercase())
                    .ok_or_else(|| anyhow!("Register identifier missing"))?
                    .strip_prefix(": ")
                    .ok_or_else(|| anyhow!("Register colon missing"))?
                    .parse::<u64>()?)
            }))?
            .map_err(|_| anyhow!("Wrong number of registers given"))?;
        let program = program_str
            .strip_prefix("Program: ")
            .ok_or_else(|| anyhow!("Program introducer missing"))?
            .split(',')
            .map(|s| s.parse())
            .try_collect()?;
        Ok(Self {
            initial_registers,
            program,
        })
    }

    fn part1(&self) -> String {
        self.run().join(",")
    }

    fn part2(&self) -> Result<u64> {
        let a = reversed_self_replication_value(&self.program);
        let test_computer = Computer {
            ip: 0,
            registers: [a, 0, 0],
            program: &self.program,
        };

        if test_computer.eq(self.program.iter().map(|&x| x as u64)) {
            Ok(a)
        } else {
            Err(anyhow!("self-consistency check failed"))
        }
    }
}

impl Door {
    fn run(&self) -> Computer<'_> {
        Computer {
            ip: 0,
            registers: self.initial_registers,
            program: &self.program,
        }
    }
}

const ADV: u8 = 0;
const BXL: u8 = 1;
const BST: u8 = 2;
const JNZ: u8 = 3;
const BXC: u8 = 4;
const OUT: u8 = 5;
const BDV: u8 = 6;
const CDV: u8 = 7;

impl Iterator for Computer<'_> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some(&[op, arg]) = self.program.get(self.ip..self.ip + 2) else {
                return None;
            };

            self.ip += 2;

            match op {
                ADV => {
                    self.registers[0] >>= self.combo(arg);
                }
                BXL => {
                    self.registers[1] ^= arg as u64;
                }
                BST => {
                    self.registers[1] = self.combo(arg) % 8;
                }
                JNZ if self.registers[0] == 0 => {}
                JNZ => {
                    self.ip = arg as usize;
                }
                BXC => {
                    self.registers[1] ^= self.registers[2];
                }
                OUT => return Some(self.combo(arg) % 8),
                BDV => {
                    self.registers[1] = self.registers[0] >> self.combo(arg);
                }
                CDV => {
                    self.registers[2] = self.registers[0] >> self.combo(arg);
                }
                8.. => unreachable!("invalid opcode"),
            }
        }
    }
}

impl Computer<'_> {
    fn combo(&self, arg: u8) -> u64 {
        match arg {
            0..4 => arg as u64,
            4..7 => self.registers[arg as usize - 4],
            7.. => unreachable!("invalid combo operand"),
        }
    }
}

#[allow(dead_code)]
fn brute_force_self_replication_value(program: &[u8]) -> u64 {
    (0..)
        .find(|&reg_a| {
            Computer {
                ip: 0,
                registers: [reg_a, 0, 0],
                program,
            }
            .eq(program.iter().map(|&p| p as u64))
        })
        .unwrap()
}

#[allow(dead_code)]
fn run_reverse_engineered_program(initial_registers: [u64; 3]) -> impl Iterator<Item = u64> {
    iterate(initial_registers, |&[mut a, _, _]| {
        let mut b = a % 8; // bst a
        b ^= 3; // bxl 3
        let c = a >> b; // cdv b
        b ^= c; // bxc _
        a >>= 3; // adv 3
        b ^= 5; // bxl 5
        [a, b, c]
    })
    .skip(1)
    .take_while_inclusive(|&[a, _, _]| a != 0) // jnz 0
    .map(|[_, b, _]| b % 8) // out b
}

fn reversed_self_replication_value(program: &[u8]) -> u64 {
    program
        .iter()
        .rev()
        .fold(vec![0], |possible_a, b4| {
            possible_a
                .iter()
                .flat_map(|a| {
                    let b3 = *b4 as u64 ^ 5;
                    (0..8).filter_map(move |b2| {
                        let b1 = b2 ^ 3;
                        let a2 = (a << 3) | b1;
                        let c = a2 >> b2;
                        (b3 == b2 ^ (c % 8)).then_some(a2)
                    })
                })
                .collect()
        })
        .into_iter()
        .min()
        .expect("should be possible for some A")
}

#[cfg(test)]
mod tests {
    use itertools::assert_equal;
    use proptest::proptest;

    use super::*;

    const EXAMPLE_INPUT: &str = "\
Register A: 729
Register B: 0
Register C: 0

Program: 0,1,5,4,3,0";

    const EXAMPLE_INIT_REG: [u64; 3] = [729, 0, 0];
    const EXAMPLE_PROGRAM: &[u8] = &[0, 1, 5, 4, 3, 0];
    const SELF_REPL_PROGRAM: &[u8] = &[0, 3, 5, 4, 3, 0];
    const INPUT_PROGRAM: &[u8] = &[2, 4, 1, 3, 7, 5, 4, 2, 0, 3, 1, 5, 5, 5, 3, 0];

    #[test]
    fn parse_example_input() {
        let door = Door::parse(EXAMPLE_INPUT).unwrap();
        assert_eq!(door.initial_registers, EXAMPLE_INIT_REG);
        assert_eq!(door.program, EXAMPLE_PROGRAM);
    }

    #[test]
    fn computer_produces_example_output() {
        let computer = Computer {
            ip: 0,
            registers: EXAMPLE_INIT_REG,
            program: EXAMPLE_PROGRAM,
        };
        assert_equal(computer, [4, 6, 3, 5, 6, 3, 5, 2, 1, 0]);
    }

    #[test]
    fn program_self_replicates_for_magic_value() {
        let computer = Computer {
            ip: 0,
            registers: [117440, 0, 0],
            program: SELF_REPL_PROGRAM,
        };
        assert_equal(computer, SELF_REPL_PROGRAM.iter().map(|&p| p as u64));
    }

    #[test]
    fn find_lowest_self_replication_value() {
        assert_eq!(
            brute_force_self_replication_value(SELF_REPL_PROGRAM),
            117440
        );
    }

    proptest! {

        #[test]
        fn reverse_engineered_function_produces_same_results(a in 0..=u64::MAX) {
            let control_computer = Computer {
                ip: 0,
                registers: [a, 0, 0],
                program: INPUT_PROGRAM,
            };
            assert_equal(run_reverse_engineered_program([a, 0, 0]), control_computer);
        }

    }

    #[test]
    fn reverse_engineered_self_replication_value_replicates_program() {
        let a = reversed_self_replication_value(INPUT_PROGRAM);
        assert_equal(
            run_reverse_engineered_program([a, 0, 0]),
            INPUT_PROGRAM.iter().map(|p| *p as u64),
        );
    }
}
