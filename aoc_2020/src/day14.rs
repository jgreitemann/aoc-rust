use std::{collections::HashMap, fmt::Write};

use anyhow::Context as _;
use aoc_companion::prelude::*;
use itertools::Itertools as _;

pub(crate) struct Door {
    program: Vec<Instruction>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Door> {
        Ok(Door {
            program: input.lines().map(|instr| instr.parse()).try_collect()?,
        })
    }

    fn part1(&self) -> u64 {
        total_memory(generate_writes::<DecoderV1>(self.program.iter().copied()))
    }

    fn part2(&self) -> u64 {
        total_memory(generate_writes::<DecoderV2>(self.program.iter().copied()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Instruction {
    SetMask(Mask),
    Write(MemWrite),
}

impl std::str::FromStr for Instruction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let Some((lhs, rhs)) = s.split_once('=') else {
            anyhow::bail!("instruction does not contain assignment operator '='");
        };
        let lhs = lhs.trim_end();
        let rhs = rhs.trim_start();
        let (introducer, rest) = lhs.split_at_checked(4).unwrap_or((lhs, ""));

        Ok(match introducer {
            "mask" => Instruction::SetMask(rhs.parse()?),
            "mem[" => Instruction::Write(MemWrite {
                addr: rest
                    .strip_suffix(']')
                    .with_context(|| "missing closing bracket")?
                    .parse()
                    .with_context(|| "invalid memory address")?,
                value: rhs.parse().with_context(|| "invalid value to write")?,
            }),
            _ => anyhow::bail!("illegal instruction, introduced by {introducer:?}"),
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Mask([Option<bool>; 36]);

impl std::str::FromStr for Mask {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        // FIXME: This should use `try_from_iter_exact()`
        aoc_utils::array::try_from_iter(s.chars().rev().map(|c| match c {
            'X' => Ok(None),
            '0' => Ok(Some(false)),
            '1' => Ok(Some(true)),
            _ => Err(anyhow::anyhow!("invalid mask character: {c:?}")),
        }))?
        .map_err(|v| anyhow::anyhow!("expected mask of length 36, got length {}", v.len()))
        .map(Mask)
    }
}

impl Default for Mask {
    fn default() -> Self {
        Self([None; 36])
    }
}

impl std::fmt::Debug for Mask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct MaskInner<'a>(&'a [Option<bool>]);

        impl std::fmt::Debug for MaskInner<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                for b in self.0.iter().rev() {
                    f.write_char(match b {
                        None => 'X',
                        Some(false) => '0',
                        Some(true) => '1',
                    })?;
                }
                Ok(())
            }
        }

        f.debug_tuple("Mask").field(&MaskInner(&self.0)).finish()
    }
}

impl Mask {
    fn bits(&self) -> impl Iterator<Item = (Option<bool>, u64)> {
        self.0.iter().copied().enumerate().map(|(i, b)| (b, 1 << i))
    }
}

trait ApplyMask {
    fn apply(mask: Mask, mem_write: MemWrite) -> impl Iterator<Item = MemWrite>;
}

struct DecoderV1;

impl ApplyMask for DecoderV1 {
    fn apply(mask: Mask, MemWrite { addr, value }: MemWrite) -> impl Iterator<Item = MemWrite> {
        std::iter::once_with(move || MemWrite {
            addr,
            value: mask
                .bits()
                .filter_map(|(b, i)| Some((b?, i)))
                .fold(value, |v, (b, i)| v ^ (v & i) ^ ((b as u64) * i)),
        })
    }
}

struct DecoderV2;

impl ApplyMask for DecoderV2 {
    fn apply(mask: Mask, MemWrite { addr, value }: MemWrite) -> impl Iterator<Item = MemWrite> {
        let addr = mask
            .bits()
            .filter_map(|(b, i)| (b == Some(true)).then_some(i))
            .fold(addr, std::ops::BitOr::bitor);

        let floating_bits = mask.bits().filter_map(|(b, i)| b.is_none().then_some(i));

        floating_bits
            .into_iter()
            .map(|i| [0, i])
            .multi_cartesian_product()
            .map(move |bits| bits.iter().fold(addr, |a, b| a ^ b))
            .map(move |addr| MemWrite { addr, value })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct MemWrite {
    addr: u64,
    value: u64,
}

fn generate_writes<D: ApplyMask>(
    program: impl IntoIterator<Item = Instruction>,
) -> impl Iterator<Item = MemWrite> {
    program
        .into_iter()
        .scan(Mask::default(), move |mask, instr| match instr {
            Instruction::SetMask(new_mask) => {
                *mask = new_mask;
                Some(None)
            }
            Instruction::Write(w) => Some(Some(D::apply(*mask, w))),
        })
        .flatten()
        .flatten()
}

fn total_memory(mem_writes: impl IntoIterator<Item = MemWrite>) -> u64 {
    let memory: HashMap<u64, u64> = mem_writes
        .into_iter()
        .map(|MemWrite { addr, value }| (addr, value))
        .collect();
    memory.into_values().sum()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    const EXAMPLE_INPUT: &str = "mask = XXXXXXXXXXXXXXXXXXXXXXXXXXXXX1XXXX0X
mem[8] = 11
mem[7] = 101
mem[8] = 0";

    const EXAMPLE_MASK: Mask = Mask([
        None,
        Some(false),
        None,
        None,
        None,
        None,
        Some(true),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ]);

    const EXAMPLE_PROGRAM: &[Instruction] = &[
        Instruction::SetMask(EXAMPLE_MASK),
        Instruction::Write(MemWrite { addr: 8, value: 11 }),
        Instruction::Write(MemWrite {
            addr: 7,
            value: 101,
        }),
        Instruction::Write(MemWrite { addr: 8, value: 0 }),
    ];

    const EXAMPLE_MEM_WRITES_V1: &[MemWrite] = &[
        MemWrite { addr: 8, value: 73 },
        MemWrite {
            addr: 7,
            value: 101,
        },
        MemWrite { addr: 8, value: 64 },
    ];

    #[test]
    fn mask_is_parsed_and_printed_consistently() {
        const EXAMPLE_MASK_STR: &str = "XXXXXXXXXXXXXXXXXXXXXXXXXXXXX1XXXX0X";
        assert_eq!(
            format!("{:?}", EXAMPLE_MASK_STR.parse::<Mask>().unwrap()),
            format!("Mask({EXAMPLE_MASK_STR})")
        );
    }

    #[test]
    fn parse_example_program() {
        itertools::assert_equal(
            Door::parse(EXAMPLE_INPUT).unwrap().program.iter(),
            EXAMPLE_PROGRAM,
        );
    }

    #[test]
    fn apply_mask_v1() {
        itertools::assert_equal(
            DecoderV1::apply(EXAMPLE_MASK, MemWrite { addr: 8, value: 11 }),
            [MemWrite { addr: 8, value: 73 }],
        );
        itertools::assert_equal(
            DecoderV1::apply(
                EXAMPLE_MASK,
                MemWrite {
                    addr: 7,
                    value: 101,
                },
            ),
            [MemWrite {
                addr: 7,
                value: 101,
            }],
        );
        itertools::assert_equal(
            DecoderV1::apply(EXAMPLE_MASK, MemWrite { addr: 8, value: 0 }),
            [MemWrite { addr: 8, value: 64 }],
        );
    }

    #[test]
    fn generate_example_writes_v1() {
        itertools::equal(
            generate_writes::<DecoderV1>(EXAMPLE_PROGRAM.iter().copied()),
            EXAMPLE_MEM_WRITES_V1.iter().copied(),
        );
    }

    #[test]
    fn find_total_memory_for_example_v1() {
        assert_eq!(total_memory(EXAMPLE_MEM_WRITES_V1.iter().copied()), 165);
    }

    #[test]
    fn apply_mask_v2() {
        let first_mask: Mask = "000000000000000000000000000000X1001X".parse().unwrap();
        assert_eq!(
            HashSet::from_iter(DecoderV2::apply(
                first_mask,
                MemWrite {
                    addr: 42,
                    value: 100,
                },
            )),
            HashSet::from([
                MemWrite {
                    addr: 26,
                    value: 100,
                },
                MemWrite {
                    addr: 27,
                    value: 100,
                },
                MemWrite {
                    addr: 58,
                    value: 100,
                },
                MemWrite {
                    addr: 59,
                    value: 100,
                },
            ]),
        );
    }

    #[test]
    fn find_total_memory_for_example_v2() {
        let Door { program } = Door::parse(
            "mask = 000000000000000000000000000000X1001X
mem[42] = 100
mask = 00000000000000000000000000000000X0XX
mem[26] = 1",
        )
        .unwrap();
        assert_eq!(total_memory(generate_writes::<DecoderV2>(program)), 208);
    }
}
