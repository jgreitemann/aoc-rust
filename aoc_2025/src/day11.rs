use std::collections::{HashMap, HashSet};

use anyhow::bail;
use aoc_companion::prelude::*;
use itertools::Itertools as _;

pub(crate) struct Door {
    connections: HashMap<Device, HashSet<Device>>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        input
            .lines()
            .map(|line| {
                let Some((this_name, connected_names)) = line.split_once(": ") else {
                    bail!("expected \": \" delimiter, found {line:?}");
                };
                let connected_devices = connected_names
                    .split_ascii_whitespace()
                    .map(Device::new)
                    .try_collect()?;
                Ok((Device::new(this_name)?, connected_devices))
            })
            .try_collect()
            .map(|connections| Door { connections })
    }

    fn part1(&self) -> usize {
        number_of_paths(YOU, &self.connections)
    }

    fn part2(&self) -> usize {
        number_of_problematic_paths(SVR, &self.connections).num_both
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Device([u8; 3]);

impl std::fmt::Debug for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = str::from_utf8(&self.0).expect("device names should be valid UTF-8");
        f.debug_tuple("Device").field(&s).finish()
    }
}

impl Device {
    fn new(name: &str) -> Result<Self> {
        if name.len() != 3 {
            bail!(
                "device name {name:?} needs to be exactly three bytes, but is {} bytes: {:x?}",
                name.len(),
                name.as_bytes()
            );
        }
        let mut bytes = [0; 3];
        bytes.copy_from_slice(name.as_bytes());
        Ok(Self(bytes))
    }
}

const OUT: Device = Device(*b"out");
const YOU: Device = Device(*b"you");
const SVR: Device = Device(*b"svr");
const DAC: Device = Device(*b"dac");
const FFT: Device = Device(*b"fft");

fn number_of_paths(from: Device, connections: &HashMap<Device, HashSet<Device>>) -> usize {
    if from == OUT {
        return 1;
    }

    connections[&from]
        .iter()
        .map(|&to| number_of_paths(to, connections))
        .sum()
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct PathStats {
    num_neither: usize,
    num_only_dac: usize,
    num_only_fft: usize,
    num_both: usize,
}

impl std::iter::Sum for PathStats {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), |acc, res| Self {
            num_neither: acc.num_neither + res.num_neither,
            num_only_dac: acc.num_only_dac + res.num_only_dac,
            num_only_fft: acc.num_only_fft + res.num_only_fft,
            num_both: acc.num_both + res.num_both,
        })
    }
}

fn number_of_problematic_paths(
    start: Device,
    connections: &HashMap<Device, HashSet<Device>>,
) -> PathStats {
    aoc_utils::cache::cached(|from, recurse| {
        if from == OUT {
            return PathStats {
                num_neither: 1,
                ..Default::default()
            };
        }

        connections[&from]
            .iter()
            .map(|&to| recurse(to))
            .map(|stats| match from {
                DAC => PathStats {
                    num_neither: 0,
                    num_only_dac: stats.num_neither,
                    num_only_fft: 0,
                    num_both: stats.num_only_fft,
                },
                FFT => PathStats {
                    num_neither: 0,
                    num_only_dac: 0,
                    num_only_fft: stats.num_neither,
                    num_both: stats.num_only_dac,
                },
                _ => stats,
            })
            .sum()
    })(start)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIRST_EXAMPLE_INPUT: &str = "\
aaa: you hhh
you: bbb ccc
bbb: ddd eee
ccc: ddd eee fff
ddd: ggg
eee: out
fff: out
ggg: out
hhh: ccc fff iii
iii: out";

    fn first_example_connections() -> HashMap<Device, HashSet<Device>> {
        HashMap::from([
            (
                Device(*b"aaa"),
                HashSet::from([Device(*b"you"), Device(*b"hhh")]),
            ),
            (
                Device(*b"you"),
                HashSet::from([Device(*b"bbb"), Device(*b"ccc")]),
            ),
            (
                Device(*b"bbb"),
                HashSet::from([Device(*b"ddd"), Device(*b"eee")]),
            ),
            (
                Device(*b"ccc"),
                HashSet::from([Device(*b"ddd"), Device(*b"eee"), Device(*b"fff")]),
            ),
            (Device(*b"ddd"), HashSet::from([Device(*b"ggg")])),
            (Device(*b"eee"), HashSet::from([Device(*b"out")])),
            (Device(*b"fff"), HashSet::from([Device(*b"out")])),
            (Device(*b"ggg"), HashSet::from([Device(*b"out")])),
            (
                Device(*b"hhh"),
                HashSet::from([Device(*b"ccc"), Device(*b"fff"), Device(*b"iii")]),
            ),
            (Device(*b"iii"), HashSet::from([Device(*b"out")])),
        ])
    }

    const SECOND_EXAMPLE_INPUT: &str = "\
svr: aaa bbb
aaa: fft
fft: ccc
bbb: tty
tty: ccc
ccc: ddd eee
ddd: hub
hub: fff
eee: dac
dac: fff
fff: ggg hhh
ggg: out
hhh: out";

    #[test]
    fn parse_example_input() {
        assert_eq!(
            Door::parse(FIRST_EXAMPLE_INPUT).unwrap().connections,
            first_example_connections(),
        );
    }

    #[test]
    fn find_number_of_paths_to_out() {
        assert_eq!(number_of_paths(YOU, &first_example_connections()), 5);
    }

    #[test]
    fn find_number_of_problematic_paths_to_out() {
        let Door { connections } = Door::parse(SECOND_EXAMPLE_INPUT).unwrap();
        assert_eq!(
            number_of_problematic_paths(SVR, &connections),
            PathStats {
                num_neither: 2,
                num_only_dac: 2,
                num_only_fft: 2,
                num_both: 2
            }
        )
    }
}
