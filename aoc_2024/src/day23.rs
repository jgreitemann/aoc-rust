use std::collections::{BTreeSet, HashMap, HashSet};

use anyhow::anyhow;
use aoc_companion::prelude::*;
use itertools::Itertools;
use tap::Tap;

pub(crate) struct Door {
    connections: Network,
}

type Network = HashMap<String, HashSet<String>>;

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        Ok(Door {
            connections: parse_connections(input)?,
        })
    }

    fn part1(&self) -> usize {
        subnets_in_question(&self.connections).count()
    }

    fn part2(&self) -> String {
        max_subnet(&self.connections).iter().join(",")
    }
}

fn parse_connections(input: &str) -> Result<Network> {
    input
        .lines()
        .map(|line| {
            line.split_once('-')
                .ok_or(anyhow!("Missing dash between connected computers"))
        })
        .map_ok(|(lhs, rhs)| {
            [
                (lhs.to_string(), rhs.to_string()),
                (rhs.to_string(), lhs.to_string()),
            ]
        })
        .flatten_ok()
        .try_fold(HashMap::new(), |mut network, res| {
            let (lhs, rhs) = res?;
            network
                .entry(lhs)
                .and_modify(|c: &mut HashSet<String>| {
                    c.insert(rhs.clone());
                })
                .or_insert(HashSet::from([rhs]));
            Ok(network)
        })
}

fn triple_subnets(network: &Network) -> HashSet<[&str; 3]> {
    let mut subnets = HashSet::new();
    for (a, connections) in network {
        for b in connections {
            for c in network.get(b).unwrap() {
                if connections.contains(c) {
                    subnets.insert(
                        [a.as_str(), b.as_str(), c.as_str()].tap_mut(|triple| triple.sort()),
                    );
                }
            }
        }
    }

    subnets
}

fn subnets_in_question(network: &Network) -> impl Iterator<Item = [&str; 3]> {
    triple_subnets(network)
        .into_iter()
        .filter(|subnet| subnet.iter().any(|c| c.starts_with('t')))
}

fn max_subnet(network: &Network) -> BTreeSet<&str> {
    max_subnet_recursive(
        network.keys().map(|s| s.as_str()).collect(),
        BTreeSet::new(),
        network,
    )
}

fn max_subnet_recursive<'n>(
    nodes: HashSet<&'n str>,
    cluster: BTreeSet<&'n str>,
    network: &'n Network,
) -> BTreeSet<&'n str> {
    let mut visited: HashSet<&str> = HashSet::new();
    nodes
        .iter()
        .flat_map(|&node| {
            if visited.contains(node) {
                return None;
            }
            let connections = network.get(node).unwrap();
            let mut new_cluster = cluster.clone();
            new_cluster.insert(node);
            let new_nodes = nodes
                .iter()
                .cloned()
                .filter(|&n| connections.contains(n))
                .filter(|&n| !cluster.contains(n))
                .collect::<HashSet<_>>();

            Some(
                if new_nodes.is_empty() {
                    new_cluster
                } else {
                    max_subnet_recursive(new_nodes, new_cluster, network)
                }
                .tap(|v| visited.extend(v)),
            )
        })
        .max_by_key(|n| n.len())
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "\
kh-tc
qp-kh
de-cg
ka-co
yn-aq
qp-ub
cg-tb
vc-aq
tb-ka
wh-tc
yn-cg
kh-ub
ta-co
de-co
tc-td
tb-wq
wh-td
ta-ka
td-qp
aq-cg
wq-ub
ub-vc
de-ta
wq-aq
wq-vc
wh-yn
ka-de
kh-ta
co-tc
wh-qp
tb-vc
td-yn";

    #[test]
    fn example_subnets_in_question() {
        let network = parse_connections(EXAMPLE_INPUT).unwrap();
        let subnets: HashSet<_> = subnets_in_question(&network).collect();
        assert_eq!(
            subnets,
            HashSet::from([
                ["co", "de", "ta"],
                ["co", "ka", "ta"],
                ["de", "ka", "ta"],
                ["qp", "td", "wh"],
                ["tb", "vc", "wq"],
                ["tc", "td", "wh"],
                ["td", "wh", "yn"],
            ]),
        )
    }

    #[test]
    fn largest_fully_connected_subnet() {
        let network = parse_connections(EXAMPLE_INPUT).unwrap();
        assert_eq!(
            max_subnet(&network),
            BTreeSet::from(["co", "de", "ka", "ta"])
        );
    }
}
