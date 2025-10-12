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
}
