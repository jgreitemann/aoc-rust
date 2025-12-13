use std::ops::BitXor;

use anyhow::{Context, anyhow, bail};
use aoc_companion::prelude::*;
use itertools::Itertools as _;
use rayon::iter::{ParallelBridge as _, ParallelIterator as _};

pub(crate) struct Door {
    machines: Vec<Machine>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        input
            .lines()
            .map(str::parse)
            .try_collect()
            .map(|machines| Door { machines })
    }

    fn part1(&self) -> Result<usize> {
        self.machines
            .iter()
            .map(|m| {
                m.fewest_button_presses_for_lights().ok_or_else(|| {
                    anyhow!("no combination of button presses achieved desired indicator lights")
                })
            })
            .try_fold(0, |acc, b| Ok(acc + b?))
    }

    fn part2(&self) -> Result<usize> {
        self.machines
            .iter()
            .map(|m| {
                m.fewest_button_presses_for_joltage().ok_or_else(|| {
                    anyhow!(
                        "no combination of button presses achieved desired joltage requirements"
                    )
                })
            })
            .try_fold(0, |acc, b| Ok(acc + b?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Machine {
    desired_indicator_lights: u32,
    buttons: Vec<u32>,
    joltage_requirements: Vec<usize>,
}

impl std::str::FromStr for Machine {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut words = s.split_ascii_whitespace();
        let Some(light_pattern) = words.next() else {
            bail!("expected indicator light pattern");
        };
        let Some(light_pattern) = light_pattern
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'))
        else {
            bail!("indicator light pattern needs to be surrounded by square brackets");
        };
        let desired_indicator_lights = light_pattern
            .bytes()
            .rev()
            .fold(0, |acc, b| (acc << 1) | (b == b'#') as u32);
        let Some(joltage_seq) = words.next_back() else {
            bail!("expected joltage requirements");
        };
        let Some(joltage_seq) = joltage_seq
            .strip_prefix('{')
            .and_then(|rest| rest.strip_suffix('}'))
        else {
            bail!("joltage requiements need to be surrounded by curly brackets");
        };
        let joltage_requirements = joltage_seq
            .split(',')
            .map(|s| {
                s.parse()
                    .with_context(|| anyhow!("parsing joltage requirement {s:?} failed"))
            })
            .try_collect()?;
        let buttons = words
            .map(|button_word| {
                let Some(button_word) = button_word
                    .strip_prefix('(')
                    .and_then(|rest| rest.strip_suffix(')'))
                else {
                    bail!("button wiring needs to be surrounded by parentheses");
                };
                button_word
                    .split(',')
                    .map(|s| {
                        s.parse()
                            .with_context(|| anyhow!("parsing button wiring {s:?} failed"))
                    })
                    .try_fold(0, |acc, i: Result<usize, _>| Ok(acc | (1 << i?)))
            })
            .try_collect()?;

        Ok(Machine {
            desired_indicator_lights,
            buttons,
            joltage_requirements,
        })
    }
}

fn button_presses_for_lights(
    buttons: &[u32],
    desired_indicator_lights: u32,
) -> impl Iterator<Item = Vec<&u32>> {
    (0..=buttons.len())
        .flat_map(|k| buttons.iter().combinations(k))
        .filter(move |combo| {
            combo.iter().copied().fold(0, BitXor::bitxor) == desired_indicator_lights
        })
}

impl Machine {
    fn fewest_button_presses_for_lights(&self) -> Option<usize> {
        button_presses_for_lights(&self.buttons, self.desired_indicator_lights)
            .map(|combo| combo.len())
            .next()
    }

    fn fewest_button_presses_for_joltage(&self) -> Option<usize> {
        fn inner(buttons: &[u32], required_joltage: &[usize]) -> Option<usize> {
            let equiv_lights = required_joltage
                .iter()
                .rev()
                .fold(0, |acc, n| (acc << 1) | (n % 2) as u32);

            button_presses_for_lights(buttons, equiv_lights)
                .par_bridge()
                .filter_map(|combo| {
                    let remaining_joltage: Vec<_> = required_joltage
                        .iter()
                        .enumerate()
                        .map(|(i, j)| {
                            j.checked_sub(
                                combo.iter().copied().filter(|&b| b & (1 << i) != 0).count(),
                            )
                            .map(|j| j / 2)
                            .ok_or(())
                        })
                        .try_collect()
                        .ok()?;

                    if remaining_joltage.iter().all(|&j| j == 0) {
                        Some(combo.len())
                    } else {
                        Some(combo.len() + 2 * inner(buttons, &remaining_joltage)?)
                    }
                })
                .min()
        }

        inner(&self.buttons, &self.joltage_requirements)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "\
[.##.] (3) (1,3) (2) (2,3) (0,2) (0,1) {3,5,4,7}
[...#.] (0,2,3,4) (2,3) (0,4) (0,1,2) (1,2,3,4) {7,5,12,7,2}
[.###.#] (0,1,2,3,4) (0,3,4) (0,1,2,4,5) (1,2) {10,11,11,5,10,5}";

    fn example_machines() -> Vec<Machine> {
        vec![
            Machine {
                desired_indicator_lights: 0b0110,
                buttons: vec![0b1000, 0b1010, 0b0100, 0b1100, 0b0101, 0b0011],
                joltage_requirements: vec![3, 5, 4, 7],
            },
            Machine {
                desired_indicator_lights: 0b01000,
                buttons: vec![0b11101, 0b01100, 0b10001, 0b00111, 0b11110],
                joltage_requirements: vec![7, 5, 12, 7, 2],
            },
            Machine {
                desired_indicator_lights: 0b101110,
                buttons: vec![0b011111, 0b011001, 0b110111, 0b000110],
                joltage_requirements: vec![10, 11, 11, 5, 10, 5],
            },
        ]
    }

    #[test]
    fn parse_example_input() {
        itertools::assert_equal(
            Door::parse(EXAMPLE_INPUT).unwrap().machines,
            example_machines(),
        );
    }

    #[test]
    fn fewest_button_presses_for_lights_in_example() {
        itertools::assert_equal(
            example_machines()
                .iter()
                .map(Machine::fewest_button_presses_for_lights),
            [Some(2), Some(3), Some(2)],
        );
    }

    #[test]
    fn fewest_button_presses_for_joltage_in_example() {
        itertools::assert_equal(
            example_machines()
                .iter()
                .map(Machine::fewest_button_presses_for_joltage),
            [Some(10), Some(12), Some(11)],
        );
    }
}
