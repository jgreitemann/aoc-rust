use aoc_companion::prelude::*;

use itertools::Itertools;

use std::collections::{HashMap, VecDeque};
use std::num::ParseIntError;

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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct ValveId(u16);

impl std::fmt::Debug for ValveId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}{}",
            char::from_u32(self.0 as u32 / 256).unwrap(),
            char::from_u32(self.0 as u32 % 256).unwrap()
        ))
    }
}

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
    connections: HashMap<ValveId, u32>,
}

type Cave = HashMap<ValveId, Valve>;

#[derive(Debug, Clone)]
struct Strategy {
    current: ValveId,
    flow: u32,
    flow_rate: u32,
    time: u32,
}

impl Strategy {
    fn new() -> Self {
        Self {
            current: "AA".into(),
            flow: 0,
            flow_rate: 0,
            time: 0,
        }
    }

    fn traverse_fully_connected_graph(mut self, final_time: u32, fc_cave: Cave) -> Strategy {
        let Valve {
            flow_rate,
            connections,
        } = &fc_cave[&self.current];

        self.flow_rate += flow_rate;

        connections
            .iter()
            .filter(|&(_, dist)| self.time + dist < final_time)
            .map(|(target, dist)| {
                let mut new_strat = self.clone();
                new_strat.time += dist;
                new_strat.flow += dist * new_strat.flow_rate;
                new_strat.current = *target;
                let mut new_cave = fc_cave.clone();
                for valve in new_cave.values_mut() {
                    valve.connections.remove(target);
                }
                new_strat.traverse_fully_connected_graph(final_time, new_cave)
            })
            .reduce(|lhs_strat, rhs_strat| std::cmp::max_by_key(lhs_strat, rhs_strat, |s| s.flow))
            .unwrap_or_else(|| {
                self.flow += (final_time - self.time) * self.flow_rate;
                self.time = final_time;
                self
            })
    }
}

fn find_optimal_strategy(cave: &Cave) -> Strategy {
    Strategy::new()
        .traverse_fully_connected_graph(MINUTES, fully_connect_cave(&reduce_cave(cave.clone())))
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

fn reduce_cave(mut cave: Cave) -> Cave {
    while let Some((&id, _)) = cave
        .iter()
        .find(|(&id, Valve { flow_rate, .. })| *flow_rate == 0 && id != "AA".into())
    {
        let Valve { connections, .. } = cave.remove(&id).unwrap();
        for (
            lhs_id,
            Valve {
                connections: lhs_connections,
                ..
            },
        ) in cave.iter_mut()
        {
            if let Some(lhs_dist) = lhs_connections.remove(&id) {
                for (rhs_id, rhs_dist) in connections.iter().filter(|(rhs_id, _)| rhs_id != &lhs_id)
                {
                    let combined_dist = lhs_dist + rhs_dist;
                    lhs_connections.insert(
                        *rhs_id,
                        lhs_connections
                            .get(rhs_id)
                            .map(|&old| old.min(combined_dist))
                            .unwrap_or(combined_dist),
                    );
                }
            }
        }
    }

    cave
}

fn fully_connect_cave(cave: &Cave) -> Cave {
    cave.iter()
        .map(|(start, &Valve { flow_rate, .. })| {
            let mut connections = HashMap::new();
            let mut queue = VecDeque::from([(*start, 1)]);
            while let Some((id, dist)) = queue.pop_front() {
                let Valve {
                    connections: id_connections,
                    ..
                } = &cave[&id];
                for (other, dist_from_id) in id_connections {
                    let prev = connections.get(other).copied().unwrap_or(u32::MAX);
                    let other_dist = dist + dist_from_id;
                    if other_dist < prev {
                        connections.insert(*other, other_dist);
                        queue.push_back((*other, other_dist));
                    }
                }
            }
            connections.remove(&"AA".into());
            connections.remove(start);
            (
                *start,
                Valve {
                    flow_rate,
                    connections,
                },
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    macro_rules! connect {
        ($($x:literal),+ $(,)?) => (HashMap::from([$(($x.into(), 1),)+]));
        ($(($x:literal, $y:literal)),+ $(,)?) => (HashMap::from([$(($x.into(), $y),)+]));
    }

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
        assert_eq!(find_optimal_strategy(&reduced_example_cave()).flow, 1651);
    }

    #[test]
    fn cave_graph_is_reduced() {
        assert_eq!(reduce_cave(example_cave()), reduced_example_cave());
    }

    #[test]
    fn cave_graph_is_fully_connected() {
        assert_eq!(
            fully_connect_cave(&reduced_example_cave()),
            fully_connected_example_cave()
        );
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
                    connections: connect![("BB", 1), ("DD", 1), ("JJ", 2)],
                },
            ),
            (
                "BB".into(),
                Valve {
                    flow_rate: 13,
                    connections: connect![("AA", 1), ("CC", 1)],
                },
            ),
            (
                "CC".into(),
                Valve {
                    flow_rate: 2,
                    connections: connect![("BB", 1), ("DD", 1)],
                },
            ),
            (
                "DD".into(),
                Valve {
                    flow_rate: 20,
                    connections: connect![("AA", 1), ("CC", 1), ("EE", 1)],
                },
            ),
            (
                "EE".into(),
                Valve {
                    flow_rate: 3,
                    connections: connect![("DD", 1), ("HH", 3)],
                },
            ),
            (
                "HH".into(),
                Valve {
                    flow_rate: 22,
                    connections: connect![("EE", 3)],
                },
            ),
            (
                "JJ".into(),
                Valve {
                    flow_rate: 21,
                    connections: connect![("AA", 2)],
                },
            ),
        ])
    }

    fn fully_connected_example_cave() -> Cave {
        HashMap::from([
            (
                "AA".into(),
                Valve {
                    flow_rate: 0,
                    connections: connect![
                        ("BB", 2),
                        ("CC", 3),
                        ("DD", 2),
                        ("EE", 3),
                        ("HH", 6),
                        ("JJ", 3)
                    ],
                },
            ),
            (
                "BB".into(),
                Valve {
                    flow_rate: 13,
                    connections: connect![("CC", 2), ("DD", 3), ("EE", 4), ("HH", 7), ("JJ", 4)],
                },
            ),
            (
                "CC".into(),
                Valve {
                    flow_rate: 2,
                    connections: connect![("BB", 2), ("DD", 2), ("EE", 3), ("HH", 6), ("JJ", 5)],
                },
            ),
            (
                "DD".into(),
                Valve {
                    flow_rate: 20,
                    connections: connect![("BB", 3), ("CC", 2), ("EE", 2), ("HH", 5), ("JJ", 4)],
                },
            ),
            (
                "EE".into(),
                Valve {
                    flow_rate: 3,
                    connections: connect![("BB", 4), ("CC", 3), ("DD", 2), ("HH", 4), ("JJ", 5)],
                },
            ),
            (
                "HH".into(),
                Valve {
                    flow_rate: 22,
                    connections: connect![("BB", 7), ("CC", 6), ("DD", 5), ("EE", 4), ("JJ", 8)],
                },
            ),
            (
                "JJ".into(),
                Valve {
                    flow_rate: 21,
                    connections: connect![("BB", 4), ("CC", 5), ("DD", 4), ("EE", 5), ("HH", 8)],
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
