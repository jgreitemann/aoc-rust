use anyhow::{Context, anyhow, bail};
use aoc_companion::prelude::*;
use itertools::Itertools as _;
use std::str::FromStr;

pub(crate) struct Door {
    arrival_time: u64,
    bus_schedule: Vec<BusLine>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        let Some((arrival_line, buses_line)) = input.split_once('\n') else {
            bail!(
                "expected exactly two lines, got {} lines",
                input.lines().count()
            )
        };

        Ok(Self {
            arrival_time: arrival_line.parse().context("invalid arrival time")?,
            bus_schedule: buses_line.split(',').map(str::parse).try_collect()?,
        })
    }

    fn part1(&self) -> Result<u64> {
        earliest_departure(self.arrival_time, &self.bus_schedule)
            .ok_or_else(|| anyhow!("no buses scheduled"))
    }

    fn part2(&self) -> Result<i128> {
        earliest_alignment(&self.bus_schedule).ok_or_else(|| anyhow!("no buses scheduled"))
    }
}

enum BusLine {
    BusId(u64),
    Omission,
}

impl FromStr for BusLine {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "x" => BusLine::Omission,
            _ => BusLine::BusId(
                s.parse()
                    .context("bus line must be either 'x' or a positive number")?,
            ),
        })
    }
}

impl BusLine {
    fn id(&self) -> Option<u64> {
        match self {
            BusLine::BusId(id) => Some(*id),
            BusLine::Omission => None,
        }
    }
}

fn earliest_departure(arrival: u64, schedule: &[BusLine]) -> Option<u64> {
    schedule
        .iter()
        .filter_map(|bus| bus.id())
        .min_by_key(|id| id - arrival % id)
        .map(|min_id| min_id * (min_id - arrival % min_id))
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Orbit {
    period: i128,
    phase: i128,
}

fn combine_orbits(a: Orbit, b: Orbit) -> Orbit {
    use num_integer::ExtendedGcd;
    use num_integer::Integer as _;
    let (ExtendedGcd { gcd, x, y: _ }, lcm) = i128::extended_gcd_lcm(&a.period, &b.period);
    let (z, 0) = (a.phase - b.phase).div_rem(&gcd) else {
        panic!("orbits {a:?} and {b:?} never align");
    };
    Orbit {
        period: lcm,
        phase: (-z * x * a.period + a.phase).rem_euclid(lcm),
    }
}

fn earliest_alignment(schedule: &[BusLine]) -> Option<i128> {
    schedule
        .iter()
        .enumerate()
        .filter_map(|(idx, bus)| {
            Some(Orbit {
                period: bus.id()? as i128,
                phase: idx as i128,
            })
        })
        .reduce(combine_orbits)
        .map(|o| (-o.phase).rem_euclid(o.period))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_ARRIVAL: u64 = 939;
    const EXAMPLE_SCHEDULE: [BusLine; 8] = [
        BusLine::BusId(7),
        BusLine::BusId(13),
        BusLine::Omission,
        BusLine::Omission,
        BusLine::BusId(59),
        BusLine::Omission,
        BusLine::BusId(31),
        BusLine::BusId(19),
    ];

    #[test]
    fn find_earliest_bus_in_example() {
        assert_eq!(
            earliest_departure(EXAMPLE_ARRIVAL, &EXAMPLE_SCHEDULE),
            Some(295)
        );
    }

    #[test]
    fn combine_orbits_incrementally() {
        assert_eq!(
            combine_orbits(
                Orbit {
                    period: 3,
                    phase: 0
                },
                Orbit {
                    period: 4,
                    phase: 1
                }
            ),
            Orbit {
                period: 12,
                phase: 9 // = (-3) % 12
            }
        );
        assert_eq!(
            combine_orbits(
                Orbit {
                    period: 12,
                    phase: 9
                },
                Orbit {
                    period: 5,
                    phase: 3
                }
            ),
            Orbit {
                period: 60,
                phase: 33 // = (-27) % 60
            }
        );
    }

    #[test]
    fn find_earliest_alignment() {
        assert_eq!(
            earliest_alignment(&[
                BusLine::BusId(3),
                BusLine::BusId(4),
                BusLine::Omission,
                BusLine::BusId(5),
            ]),
            Some(27)
        );
        assert_eq!(earliest_alignment(&EXAMPLE_SCHEDULE), Some(1068781));
        assert_eq!(
            earliest_alignment(&[
                BusLine::BusId(17),
                BusLine::Omission,
                BusLine::BusId(13),
                BusLine::BusId(19),
            ]),
            Some(3417)
        );
        assert_eq!(
            earliest_alignment(&[
                BusLine::BusId(67),
                BusLine::BusId(7),
                BusLine::BusId(59),
                BusLine::BusId(61),
            ]),
            Some(754018)
        );
        assert_eq!(
            earliest_alignment(&[
                BusLine::BusId(67),
                BusLine::Omission,
                BusLine::BusId(7),
                BusLine::BusId(59),
                BusLine::BusId(61),
            ]),
            Some(779210)
        );
        assert_eq!(
            earliest_alignment(&[
                BusLine::BusId(67),
                BusLine::BusId(7),
                BusLine::Omission,
                BusLine::BusId(59),
                BusLine::BusId(61),
            ]),
            Some(1261476)
        );
        assert_eq!(
            earliest_alignment(&[
                BusLine::BusId(1789),
                BusLine::BusId(37),
                BusLine::BusId(47),
                BusLine::BusId(1889),
            ]),
            Some(1202161486)
        );
    }
}
