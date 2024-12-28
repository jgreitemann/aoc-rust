use std::collections::{HashMap, HashSet};

use aoc_companion::prelude::*;
use aoc_utils::{geometry::map_bounds, linalg::Vector};
use itertools::iterate;

pub(crate) struct Door {
    initially_infected: HashSet<Vector<isize, 2>>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Self {
        let center = Vector(map_bounds(input).map(|b| b.end as isize)) / 2;
        let initially_infected = input
            .lines()
            .enumerate()
            .flat_map(|(row, line)| {
                line.as_bytes()
                    .iter()
                    .enumerate()
                    .filter_map(move |(col, &b)| {
                        (b == b'#').then_some(Vector([col as isize, row as isize]) - center)
                    })
            })
            .collect();
        Self { initially_infected }
    }

    fn part1(&self) -> usize {
        carrier_part1(self.initially_infected.clone())
            .take(10000)
            .filter(|&b| b)
            .count()
    }

    fn part2(&self) -> usize {
        carrier_part2(self.initially_infected.clone())
            .take(10_000_000)
            .filter(|(_, s)| *s == Some(InfectionState::Infected))
            .count()
    }
}

fn carrier_part1(mut infected: HashSet<Vector<isize, 2>>) -> impl Iterator<Item = bool> {
    iterate(
        (false, Vector([0, 0]), Vector([0, -1])),
        move |(_, p, d)| {
            if infected.contains(p) {
                let d = Vector([-d[1], d[0]]);
                infected.remove(p);
                (false, *p + d, d)
            } else {
                let d = Vector([d[1], -d[0]]);
                infected.insert(*p);
                (true, *p + d, d)
            }
        },
    )
    .map(|(b, _, _)| b)
    .skip(1)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InfectionState {
    Weakened,
    Infected,
    Flagged,
}

fn carrier_part2(
    infected: HashSet<Vector<isize, 2>>,
) -> impl Iterator<Item = (Vector<isize, 2>, Option<InfectionState>)> {
    let mut infection_state: HashMap<_, _> = infected
        .into_iter()
        .map(|p| (p, InfectionState::Infected))
        .collect();
    iterate(
        (Vector([0, 0]), Vector([0, -1]), None),
        move |&(p, d, _)| {
            let (next_p, next_d, new_s) = match infection_state.get(&p).cloned() {
                None => {
                    let d = Vector([d[1], -d[0]]);
                    (p + d, d, Some(InfectionState::Weakened))
                }
                Some(InfectionState::Weakened) => (p + d, d, Some(InfectionState::Infected)),
                Some(InfectionState::Infected) => {
                    let d = Vector([-d[1], d[0]]);
                    (p + d, d, Some(InfectionState::Flagged))
                }
                Some(InfectionState::Flagged) => {
                    let d = d * -1;
                    (p + d, d, None)
                }
            };
            if let Some(new_s) = new_s {
                infection_state.insert(p, new_s);
            } else {
                infection_state.remove(&p);
            }
            (next_p, next_d, new_s)
        },
    )
    .skip(1)
    .map(|(p, d, s)| (p - d, s))
}

#[cfg(test)]
mod tests {
    use itertools::assert_equal;

    use super::*;

    const EXAMPLE_INPUT: &str = "\
..#
#..
...";

    const EXAMPLE_INITALLY_INFECTED: [Vector<isize, 2>; 2] = [Vector([1, -1]), Vector([-1, 0])];

    #[test]
    fn parse_example_input() {
        assert_eq!(
            Door::parse(EXAMPLE_INPUT).initially_infected,
            HashSet::from(EXAMPLE_INITALLY_INFECTED),
        );
    }

    #[test]
    fn part1_infections_after_70_bursts() {
        assert_eq!(
            carrier_part1(HashSet::from(EXAMPLE_INITALLY_INFECTED))
                .take(70)
                .filter(|&b| b)
                .count(),
            41
        );
    }

    #[test]
    fn part1_initial_infection_handled_correctly() {
        assert_equal(carrier_part1(HashSet::new()).take(2), [true, true]);
        assert_equal(
            carrier_part1(HashSet::from([Vector([0, 0])])).take(2),
            [false, true],
        );
    }

    #[test]
    fn part2_infections_after_100_bursts() {
        assert_eq!(
            carrier_part2(HashSet::from(EXAMPLE_INITALLY_INFECTED))
                .take(100)
                .filter(|(_, s)| *s == Some(InfectionState::Infected))
                .count(),
            26
        );
    }

    #[test]
    fn part2_infections_after_10_000_000_bursts() {
        assert_eq!(
            carrier_part2(HashSet::from(EXAMPLE_INITALLY_INFECTED))
                .take(10_000_000)
                .filter(|(_, s)| *s == Some(InfectionState::Infected))
                .count(),
            2511944
        );
    }
}
