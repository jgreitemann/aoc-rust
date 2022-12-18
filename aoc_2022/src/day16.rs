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
        ValveId(first as u16 * 256 + second as u16)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Valve {
    flow_rate: u32,
    connections: Vec<(ValveId, u32)>,
}

type Cave = HashMap<ValveId, Valve>;

#[derive(Debug, Clone)]
enum Action {
    OpenValve(ValveId),
    MoveToTunnel { valve: ValveId, distance: u32 },
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
        match &action {
            Action::OpenValve(valve) => {
                self.time += 1;
                self.flow += self.flow_rate;
                self.flow_rate += cave[valve].flow_rate;
                self.open_valves.insert(*valve);
                self.previous = ValveId(0);
            }
            Action::MoveToTunnel { valve, distance } => {
                let delta_t = if self.time + distance >= MINUTES {
                    MINUTES - self.time
                } else {
                    *distance
                };
                self.time += delta_t;
                self.flow += self.flow_rate * delta_t;
                self.previous = self.current;
                self.current = *valve;
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
        for &(valve, distance) in current_valve
            .connections
            .iter()
            .filter(|&&(connected, _)| strat.previous != connected)
        {
            co.yield_(Action::MoveToTunnel { valve, distance }).await;
        }
    })
    .into_iter()
}

fn parse_input(input: &str) -> Result<Cave, ParseIntError> {
    let re = regex::Regex::new(r"Valve (?P<tunnel>[A-Z]+) has flow rate=(?P<rate>\d+); tunnels? leads? to valves? (?P<connections>[A-Z]+(, [A-Z]+)*)").unwrap();
    re.captures_iter(input)
        .map(|capt| {
            let connections = capt["connections"]
                .split(", ")
                .map(|id| (id.into(), 1))
                .collect();
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

fn reduce_cave(cave: Cave) -> Cave {
    cave
}

macro_rules! connect {
    ($($x:literal),+ $(,)?) => (vec![$(($x.into(), 1),)+]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_is_parsed() {
        assert_eq!(parse_input(EXAMPLE_INPUT).unwrap(), example_cave());
    }

    #[test]
    fn number_of_distinct_valve_ids() {
        assert_eq!(parse_input(REAL_INPUT).unwrap().len(), 59);
    }

    #[test]
    fn maximum_flow_rate() {
        assert_eq!(find_optimal_strategy(&example_cave()).flow, 1651);
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

    fn example_cave() -> Cave {
        HashMap::from([
            (
                "AA".into(),
                Valve {
                    flow_rate: 0,
                    connections: connect!["DD", "II", "BB"],
                },
            ),
            (
                "BB".into(),
                Valve {
                    flow_rate: 13,
                    connections: connect!["CC", "AA"],
                },
            ),
            (
                "CC".into(),
                Valve {
                    flow_rate: 2,
                    connections: connect!["DD", "BB"],
                },
            ),
            (
                "DD".into(),
                Valve {
                    flow_rate: 20,
                    connections: connect!["CC", "AA", "EE"],
                },
            ),
            (
                "EE".into(),
                Valve {
                    flow_rate: 3,
                    connections: connect!["FF", "DD"],
                },
            ),
            (
                "FF".into(),
                Valve {
                    flow_rate: 0,
                    connections: connect!["EE", "GG"],
                },
            ),
            (
                "GG".into(),
                Valve {
                    flow_rate: 0,
                    connections: connect!["FF", "HH"],
                },
            ),
            (
                "HH".into(),
                Valve {
                    flow_rate: 22,
                    connections: connect!["GG"],
                },
            ),
            (
                "II".into(),
                Valve {
                    flow_rate: 0,
                    connections: connect!["AA", "JJ"],
                },
            ),
            (
                "JJ".into(),
                Valve {
                    flow_rate: 21,
                    connections: connect!["II"],
                },
            ),
        ])
    }

    fn reduced_example_cave() -> Cave {
        HashMap::from([
            (
                "AA".into(),
                Valve {
                    flow_rate: 0,
                    connections: vec![("BB".into(), 1), ("DD".into(), 1), ("JJ".into(), 2)],
                },
            ),
            (
                "BB".into(),
                Valve {
                    flow_rate: 13,
                    connections: vec![("AA".into(), 1), ("CC".into(), 1)],
                },
            ),
            (
                "CC".into(),
                Valve {
                    flow_rate: 2,
                    connections: vec![("BB".into(), 1), ("DD".into(), 1)],
                },
            ),
            (
                "DD".into(),
                Valve {
                    flow_rate: 20,
                    connections: vec![("AA".into(), 1), ("CC".into(), 1), ("EE".into(), 1)],
                },
            ),
            (
                "EE".into(),
                Valve {
                    flow_rate: 3,
                    connections: vec![("DD".into(), 1), ("HH".into(), 3)],
                },
            ),
            (
                "HH".into(),
                Valve {
                    flow_rate: 22,
                    connections: vec![("EE".into(), 3)],
                },
            ),
            (
                "JJ".into(),
                Valve {
                    flow_rate: 21,
                    connections: vec![("AA".into(), 2)],
                },
            ),
        ])
    }

    const REAL_INPUT: &str = "\
        Valve EF has flow rate=22; tunnels lead to valves FK, HT, DE
        Valve WT has flow rate=0; tunnels lead to valves XJ, XR
        Valve RQ has flow rate=0; tunnels lead to valves VG, AV
        Valve HF has flow rate=17; tunnels lead to valves EO, PQ, GX
        Valve ZH has flow rate=0; tunnels lead to valves VG, RU
        Valve AV has flow rate=0; tunnels lead to valves RQ, VQ
        Valve AH has flow rate=12; tunnels lead to valves DF, FC, DE, MV, YC
        Valve PQ has flow rate=0; tunnels lead to valves CF, HF
        Valve DP has flow rate=18; tunnels lead to valves RD, OP, DR
        Valve RU has flow rate=16; tunnels lead to valves ZH, VJ, AQ, SG
        Valve AQ has flow rate=0; tunnels lead to valves RU, WE
        Valve KO has flow rate=0; tunnels lead to valves VQ, HQ
        Valve EY has flow rate=0; tunnels lead to valves WE, VQ
        Valve RC has flow rate=14; tunnels lead to valves QK, BL, EO
        Valve AA has flow rate=0; tunnels lead to valves XV, MS, BG, RT, HQ
        Valve IH has flow rate=0; tunnels lead to valves VQ, VJ
        Valve CK has flow rate=0; tunnels lead to valves SG, KG
        Valve BG has flow rate=0; tunnels lead to valves DY, AA
        Valve UJ has flow rate=0; tunnels lead to valves AF, OY
        Valve HQ has flow rate=0; tunnels lead to valves AA, KO
        Valve XV has flow rate=0; tunnels lead to valves AA, YL
        Valve BL has flow rate=0; tunnels lead to valves DY, RC
        Valve YL has flow rate=0; tunnels lead to valves WE, XV
        Valve RT has flow rate=0; tunnels lead to valves VG, AA
        Valve MV has flow rate=0; tunnels lead to valves AH, OM
        Valve WE has flow rate=5; tunnels lead to valves AQ, YL, OM, ZU, EY
        Valve HN has flow rate=0; tunnels lead to valves OP, XJ
        Valve UR has flow rate=0; tunnels lead to valves NZ, OY
        Valve FK has flow rate=0; tunnels lead to valves OY, EF
        Valve GE has flow rate=0; tunnels lead to valves DF, XE
        Valve GX has flow rate=0; tunnels lead to valves HF, DY
        Valve YC has flow rate=0; tunnels lead to valves QC, AH
        Valve XR has flow rate=0; tunnels lead to valves DY, WT
        Valve MS has flow rate=0; tunnels lead to valves AA, DR
        Valve EO has flow rate=0; tunnels lead to valves HF, RC
        Valve VQ has flow rate=9; tunnels lead to valves NZ, KO, EY, AV, IH
        Valve DY has flow rate=23; tunnels lead to valves XR, GX, BL, BG
        Valve XJ has flow rate=24; tunnels lead to valves QK, HN, WT
        Valve RD has flow rate=0; tunnels lead to valves VG, DP
        Valve ZU has flow rate=0; tunnels lead to valves VG, WE
        Valve AF has flow rate=0; tunnels lead to valves KG, UJ
        Valve DR has flow rate=0; tunnels lead to valves MS, DP
        Valve NZ has flow rate=0; tunnels lead to valves VQ, UR
        Valve DE has flow rate=0; tunnels lead to valves EF, AH
        Valve OP has flow rate=0; tunnels lead to valves DP, HN
        Valve QK has flow rate=0; tunnels lead to valves XJ, RC
        Valve CF has flow rate=20; tunnel leads to valve PQ
        Valve FC has flow rate=0; tunnels lead to valves KH, AH
        Valve KG has flow rate=25; tunnels lead to valves HT, AF, KH, CK
        Valve XE has flow rate=11; tunnel leads to valve GE
        Valve OY has flow rate=7; tunnels lead to valves FK, UJ, UR, QC
        Valve OM has flow rate=0; tunnels lead to valves MV, WE
        Valve QC has flow rate=0; tunnels lead to valves YC, OY
        Valve DF has flow rate=0; tunnels lead to valves AH, GE
        Valve KH has flow rate=0; tunnels lead to valves KG, FC
        Valve SG has flow rate=0; tunnels lead to valves CK, RU
        Valve VG has flow rate=3; tunnels lead to valves ZH, ZU, RQ, RD, RT
        Valve HT has flow rate=0; tunnels lead to valves KG, EF
        Valve VJ has flow rate=0; tunnels lead to valves IH, RU";
}
