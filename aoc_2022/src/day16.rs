use aoc_companion::prelude::*;

use genawaiter::sync::{Gen, GenBoxed};
use itertools::Itertools;
use tap::Tap;

use std::collections::{HashMap, HashSet};
use std::num::ParseIntError;
use std::sync::Arc;

const MINUTES: u32 = 30;

pub struct Door {
    cave: Cave,
}

impl ParseInput<'_> for Door {
    type Error = ParseIntError;

    fn parse(input: &str) -> Result<Self, Self::Error> {
        parse_input(input).map(|cave| Self { cave })
    }
}

impl Part1 for Door {
    type Output = u32;
    type Error = std::convert::Infallible;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        Ok(find_optimal_strategy(&self.cave).flow)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ValveId(u16);

impl From<&str> for ValveId {
    fn from(s: &str) -> Self {
        let &[first, second] = s.as_bytes() else {
            panic!("Valve IDs must contain two letters!");
        };
        ValveId(first as u16 * second as u16)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Valve {
    flow_rate: u32,
    connections: Vec<ValveId>,
}

type Cave = HashMap<ValveId, Valve>;

#[derive(Debug, Clone)]
enum Action {
    OpenValve(ValveId),
    MoveToTunnel(ValveId),
}

#[derive(Debug, Clone)]
struct Strategy {
    open_valves: HashSet<ValveId>,
    current: ValveId,
    previous: ValveId,
    flow: u32,
    flow_rate: u32,
    time: u32,
}

impl Strategy {
    fn new(cave: &Cave) -> Self {
        Self {
            open_valves: HashSet::from_iter(
                cave.iter()
                    .filter(|(_, valve)| valve.flow_rate == 0)
                    .map(|(id, _)| *id),
            ),
            current: "AA".into(),
            previous: ValveId(0),
            flow: 0,
            flow_rate: 0,
            time: 0,
        }
    }

    fn time_is_up(&self) -> bool {
        self.time >= MINUTES
    }

    fn all_valves_open(&self, cave: &Cave) -> bool {
        self.open_valves.len() == cave.len()
    }

    fn take(&mut self, action: Action, cave: &Cave) {
        self.time += 1;
        self.flow += self.flow_rate;
        match &action {
            Action::OpenValve(valve) => {
                self.flow_rate += cave[valve].flow_rate;
                self.open_valves.insert(*valve);
                self.previous = ValveId(0);
            }
            Action::MoveToTunnel(to) => {
                self.previous = self.current;
                self.current = *to;
            }
        }
    }
}

fn find_optimal_strategy(cave: &Cave) -> Strategy {
    strategy_dfs(Strategy::new(cave), Arc::new(cave.clone()))
        .into_iter()
        .max_by_key(|s| s.flow)
        .unwrap()
}

fn strategy_dfs(strat: Strategy, cave: Arc<Cave>) -> genawaiter::sync::GenBoxed<Strategy> {
    GenBoxed::new_boxed(|co| async move {
        for action in possible_actions(strat.clone(), &cave) {
            let mut new_strat = strat.clone().tap_mut(|s| s.take(action, &cave));
            if new_strat.time_is_up() {
                co.yield_(new_strat).await;
            } else if new_strat.all_valves_open(&cave) {
                new_strat.flow += (MINUTES - new_strat.time) * new_strat.flow_rate;
                co.yield_(new_strat).await;
            } else {
                for s in strategy_dfs(new_strat, cave.clone()) {
                    co.yield_(s).await;
                }
            }
        }
    })
}

fn possible_actions(strat: Strategy, cave: &Cave) -> impl Iterator<Item = Action> + Send + '_ {
    Gen::new(|co| async move {
        let current_valve = &cave[&strat.current];
        if current_valve.flow_rate > 0 && !strat.open_valves.contains(&strat.current) {
            co.yield_(Action::OpenValve(strat.current)).await;
        }
        for connected in current_valve
            .connections
            .iter()
            .filter(|&&connected| strat.previous != connected)
        {
            co.yield_(Action::MoveToTunnel(*connected)).await;
        }
    })
    .into_iter()
}

fn parse_input(input: &str) -> Result<Cave, ParseIntError> {
    let re = regex::Regex::new(r"Valve (?P<tunnel>[A-Z]+) has flow rate=(?P<rate>\d+); tunnels? leads? to valves? (?P<connections>[A-Z]+(, [A-Z]+)*)").unwrap();
    re.captures_iter(input)
        .map(|capt| {
            let connections = capt["connections"].split(", ").map(ValveId::from).collect();
            Ok((
                capt["tunnel"].into(),
                Valve {
                    flow_rate: capt["rate"].parse()?,
                    connections,
                },
            ))
        })
        .try_collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_is_parsed() {
        assert_eq!(parse_input(EXAMPLE_INPUT).unwrap(), example_valves());
    }

    #[test]
    fn maximum_flow_rate() {
        assert_eq!(find_optimal_strategy(&example_valves()).flow, 1651);
    }

    const EXAMPLE_INPUT: &str = "\
Valve AA has flow rate=0; tunnels lead to valves DD, II, BB
Valve BB has flow rate=13; tunnels lead to valves CC, AA
Valve CC has flow rate=2; tunnels lead to valves DD, BB
Valve DD has flow rate=20; tunnels lead to valves CC, AA, EE
Valve EE has flow rate=3; tunnels lead to valves FF, DD
Valve FF has flow rate=0; tunnels lead to valves EE, GG
Valve GG has flow rate=0; tunnels lead to valves FF, HH
Valve HH has flow rate=22; tunnel leads to valve GG
Valve II has flow rate=0; tunnels lead to valves AA, JJ
Valve JJ has flow rate=21; tunnel leads to valve II";

    fn example_valves() -> Cave {
        HashMap::from([
            (
                "AA".into(),
                Valve {
                    flow_rate: 0,
                    connections: vec!["DD".into(), "II".into(), "BB".into()],
                },
            ),
            (
                "BB".into(),
                Valve {
                    flow_rate: 13,
                    connections: vec!["CC".into(), "AA".into()],
                },
            ),
            (
                "CC".into(),
                Valve {
                    flow_rate: 2,
                    connections: vec!["DD".into(), "BB".into()],
                },
            ),
            (
                "DD".into(),
                Valve {
                    flow_rate: 20,
                    connections: vec!["CC".into(), "AA".into(), "EE".into()],
                },
            ),
            (
                "EE".into(),
                Valve {
                    flow_rate: 3,
                    connections: vec!["FF".into(), "DD".into()],
                },
            ),
            (
                "FF".into(),
                Valve {
                    flow_rate: 0,
                    connections: vec!["EE".into(), "GG".into()],
                },
            ),
            (
                "GG".into(),
                Valve {
                    flow_rate: 0,
                    connections: vec!["FF".into(), "HH".into()],
                },
            ),
            (
                "HH".into(),
                Valve {
                    flow_rate: 22,
                    connections: vec!["GG".into()],
                },
            ),
            (
                "II".into(),
                Valve {
                    flow_rate: 0,
                    connections: vec!["AA".into(), "JJ".into()],
                },
            ),
            (
                "JJ".into(),
                Valve {
                    flow_rate: 21,
                    connections: vec!["II".into()],
                },
            ),
        ])
    }
}
