use aoc_companion::prelude::*;
use itertools::{Itertools, MinMaxResult};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Input line doesn't not match regular expression: {line:?}")]
    LineDoesNotMatch { line: String },
}

#[derive(Debug, Error)]
pub enum GraphError {
    #[error("The graph is cyclic")]
    GraphIsCyclic,
    #[error("There are multiple tree components")]
    MultipleTreeComponents,
    #[error("The tower is fully balanced")]
    TowerIsBalanced,
    #[error("The minority subtower weight ({minority_weight}) was found {minority_count} times")]
    AmbiguousMinorityWeight {
        minority_weight: u32,
        minority_count: usize,
    },
}

pub struct Door<'input> {
    relations: HashMap<ProgramName<'input>, Relation<'input>>,
}

impl<'input> ParseInput<'input> for Door<'input> {
    type Error = ParseError;

    fn parse(input: &'input str) -> Result<Self, Self::Error> {
        let relations = parse_input(input)?;
        Ok(Self { relations })
    }
}

impl<'input> Part1 for Door<'input> {
    type Output = ProgramName<'input>;
    type Error = GraphError;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        find_bottom_program(&self.relations)
    }
}

impl Part2 for Door<'_> {
    type Output = u32;
    type Error = GraphError;

    fn part2(&self) -> Result<Self::Output, Self::Error> {
        let subtower_weights = calc_subtower_weights(&self.relations)?;
        correct_weight(&self.relations, &subtower_weights)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ProgramName<'input>(&'input str);

impl Display for ProgramName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

#[derive(Debug)]
struct Relation<'input> {
    weight: u32,
    decendents: HashSet<ProgramName<'input>>,
}

fn parse_input<'input>(
    input: &'input str,
) -> Result<HashMap<ProgramName<'input>, Relation<'input>>, ParseError> {
    let re = regex::Regex::new(
        r"^(?P<prog>\w+) \((?P<weight>\d+)\)(?: -> (?P<decendents>(?:\w+)(?:, (?:\w+))*))?$",
    )
    .unwrap();

    input
        .lines()
        .map(|line| {
            re.captures(line)
                .ok_or_else(|| ParseError::LineDoesNotMatch {
                    line: line.to_string(),
                })
        })
        .map_ok(|caps| {
            (
                ProgramName(caps.name("prog").unwrap().as_str()),
                Relation {
                    weight: caps["weight"].parse().unwrap(),
                    decendents: caps
                        .name("decendents")
                        .map(|decendents| {
                            decendents
                                .as_str()
                                .split(", ")
                                .map(|m| ProgramName(m))
                                .collect()
                        })
                        .unwrap_or_default(),
                },
            )
        })
        .collect()
}

fn find_bottom_program<'input>(
    relations: &HashMap<ProgramName<'input>, Relation<'input>>,
) -> Result<ProgramName<'input>, GraphError> {
    let dependent_programs: HashSet<ProgramName> = relations
        .values()
        .flat_map(|r| r.decendents.iter())
        .cloned()
        .collect();
    relations
        .keys()
        .filter(|k| !dependent_programs.contains(k))
        .cloned()
        .at_most_one()
        .map_err(|_| GraphError::MultipleTreeComponents)?
        .ok_or(GraphError::GraphIsCyclic)
}

fn tuple_op<A1, B1, C1, A2, B2, C2>(
    f1: impl Fn(A1, B1) -> C1,
    f2: impl Fn(A2, B2) -> C2,
) -> impl Fn((A1, A2), (B1, B2)) -> (C1, C2) {
    move |(a1, a2), (b1, b2)| (f1(a1, b1), f2(a2, b2))
}

fn merge<K, V>(lhs: HashMap<K, V>, rhs: HashMap<K, V>) -> HashMap<K, V>
where
    K: Eq + std::hash::Hash,
{
    let (mut source, mut sink) = if lhs.len() < rhs.len() {
        (lhs, rhs)
    } else {
        (rhs, lhs)
    };
    for (k, v) in source.drain() {
        sink.insert(k, v);
    }
    sink
}

fn subtower_weights_impl<'input>(
    relations: &HashMap<ProgramName<'input>, Relation<'input>>,
    bottom_program: ProgramName<'input>,
) -> (u32, HashMap<ProgramName<'input>, u32>) {
    let Relation { weight, decendents } = &relations[&bottom_program];
    let (subtower_weight, mut map) = decendents
        .iter()
        .map(|&prog| subtower_weights_impl(relations, prog))
        .fold(
            (*weight, HashMap::new()),
            tuple_op(std::ops::Add::add, merge),
        );
    map.insert(bottom_program, subtower_weight);
    (subtower_weight, map)
}

fn calc_subtower_weights<'input>(
    relations: &HashMap<ProgramName<'input>, Relation<'input>>,
) -> Result<HashMap<ProgramName<'input>, u32>, GraphError> {
    Ok(subtower_weights_impl(relations, find_bottom_program(relations)?).1)
}

struct SubtowerIter<'input, 'rel> {
    relations: &'rel HashMap<ProgramName<'input>, Relation<'input>>,
    current: ProgramName<'input>,
    iter: std::collections::hash_set::Iter<'rel, ProgramName<'input>>,
    subiter: Option<Box<dyn Iterator<Item = ProgramName<'input>> + 'rel>>,
}

impl<'input, 'rel> SubtowerIter<'input, 'rel> {
    fn new(
        relations: &'rel HashMap<ProgramName<'input>, Relation<'input>>,
        current: ProgramName<'input>,
    ) -> Self {
        let iter = relations[&current].decendents.iter();
        Self {
            relations,
            current,
            iter,
            subiter: None,
        }
    }
}

impl<'input> Iterator for SubtowerIter<'input, '_> {
    type Item = ProgramName<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.subiter {
            None => self.iter.next().cloned().or_else(|| {
                self.subiter = Some(Box::new(
                    self.relations[&self.current]
                        .decendents
                        .iter()
                        .map(|desc| SubtowerIter::new(self.relations, *desc))
                        .flatten(),
                ));
                self.next()
            }),
            Some(sub) => sub.next(),
        }
    }
}

fn correct_weight(
    relations: &HashMap<ProgramName, Relation>,
    subtower_weights: &HashMap<ProgramName, u32>,
) -> Result<u32, GraphError> {
    let root = find_bottom_program(relations)?;

    let subtowers = std::iter::once(root).chain(SubtowerIter::new(relations, root));

    let (weights, minority_weight, minority_count, majority_weight) = subtowers
        .map(|prog| {
            let Relation { decendents, .. } = &relations[&prog];
            (
                decendents
                    .iter()
                    .map(|desc| (subtower_weights[desc], relations[desc].weight))
                    .collect::<HashMap<_, _>>(),
                decendents.iter().counts_by(|desc| subtower_weights[desc]),
            )
        })
        .filter_map(|(weights, counts)| {
            match counts.into_iter().minmax_by_key(|(_, count)| *count) {
                MinMaxResult::NoElements | MinMaxResult::OneElement(_) => None,
                MinMaxResult::MinMax((minority_weight, minority_count), (majority_weight, _)) => {
                    Some((weights, minority_weight, minority_count, majority_weight))
                }
            }
        })
        .last()
        .ok_or(GraphError::TowerIsBalanced)?;

    match minority_count {
        1 => Ok(weights[&minority_weight] + majority_weight - minority_weight),
        _ => Err(GraphError::AmbiguousMinorityWeight {
            minority_weight,
            minority_count,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use itertools::assert_equal;
    use rstest::*;

    const EXAMPLE_INPUT: &str = r#"pbga (66)
xhth (57)
ebii (61)
havc (66)
ktlj (57)
fwft (72) -> ktlj, cntj, xhth
qoyq (66)
padx (45) -> pbga, havc, qoyq
tknk (41) -> ugml, padx, fwft
jptl (61)
ugml (68) -> gyxo, ebii, jptl
gyxo (61)
cntj (57)
"#;

    #[fixture]
    #[once]
    fn relations() -> HashMap<ProgramName<'static>, Relation<'static>> {
        parse_input(EXAMPLE_INPUT).unwrap()
    }

    #[rstest]
    #[case("pbga", 66, &[])]
    #[case("xhth", 57, &[])]
    #[case("ebii", 61, &[])]
    #[case("havc", 66, &[])]
    #[case("ktlj", 57, &[])]
    #[case("fwft", 72, &["ktlj", "cntj", "xhth"])]
    #[case("qoyq", 66, &[])]
    #[case("padx", 45, &["pbga", "havc", "qoyq"])]
    #[case("tknk", 41, &["ugml", "padx", "fwft"])]
    #[case("jptl", 61, &[])]
    #[case("ugml", 68, &["gyxo", "ebii", "jptl"])]
    #[case("gyxo", 61, &[])]
    #[case("cntj", 57, &[])]
    fn valid_input_is_parsed(
        relations: &HashMap<ProgramName, Relation>,
        #[case] prog: &str,
        #[case] exp_weight: u32,
        #[case] names: &[&str],
    ) {
        let matches = |d: &HashSet<ProgramName>, names: &[&str]| {
            names.len() == d.len() && names.into_iter().all(|name| d.contains(&ProgramName(name)))
        };
        assert_matches!(
                relations.get(&ProgramName(prog)),
                Some(Relation{ weight, decendents })
                if *weight == exp_weight && matches(decendents, names)
        );
    }

    #[rstest]
    #[case("A -> B")]
    #[case("A (42) B")]
    #[case("A (-1) -> B")]
    #[case("A (42) -> , B")]
    #[case("A (42) -> B\nfoo")]
    fn invalid_input_produces_errors(#[case] input: &str) {
        assert_matches!(parse_input(input), Err(ParseError::LineDoesNotMatch { .. }));
    }

    #[rstest]
    fn bottom_program_is_found(relations: &HashMap<ProgramName, Relation>) {
        assert_matches!(find_bottom_program(relations), Ok(ProgramName("tknk")));
    }

    #[test]
    fn bottom_program_does_not_exist_because_of_cycles() {
        let relations = HashMap::from([
            (
                ProgramName("A"),
                Relation {
                    weight: 42,
                    decendents: HashSet::from([ProgramName("B")]),
                },
            ),
            (
                ProgramName("B"),
                Relation {
                    weight: 42,
                    decendents: HashSet::from([ProgramName("C")]),
                },
            ),
            (
                ProgramName("C"),
                Relation {
                    weight: 42,
                    decendents: HashSet::from([ProgramName("A")]),
                },
            ),
        ]);
        assert_matches!(
            find_bottom_program(&relations),
            Err(GraphError::GraphIsCyclic)
        );
    }

    #[test]
    fn bottom_program_is_ambiguous_because_graph_has_multiple_components() {
        let relations = HashMap::from([
            (
                ProgramName("A"),
                Relation {
                    weight: 42,
                    decendents: HashSet::from([ProgramName("B")]),
                },
            ),
            (
                ProgramName("B"),
                Relation {
                    weight: 42,
                    decendents: HashSet::from([ProgramName("C")]),
                },
            ),
            (
                ProgramName("D"),
                Relation {
                    weight: 42,
                    decendents: HashSet::from([ProgramName("E")]),
                },
            ),
        ]);
        assert_matches!(
            find_bottom_program(&relations),
            Err(GraphError::MultipleTreeComponents)
        );
    }

    #[fixture]
    #[once]
    fn subtower_weights(
        relations: &HashMap<ProgramName<'static>, Relation<'static>>,
    ) -> HashMap<ProgramName<'static>, u32> {
        calc_subtower_weights(relations).unwrap()
    }

    #[rstest]
    #[case(ProgramName("gyxo"), 61)]
    #[case(ProgramName("jptl"), 61)]
    #[case(ProgramName("ebii"), 61)]
    #[case(ProgramName("pbga"), 66)]
    #[case(ProgramName("havc"), 66)]
    #[case(ProgramName("qoyq"), 66)]
    #[case(ProgramName("ktlj"), 57)]
    #[case(ProgramName("cntj"), 57)]
    #[case(ProgramName("xhth"), 57)]
    #[case(ProgramName("ugml"), 251)]
    #[case(ProgramName("padx"), 243)]
    #[case(ProgramName("fwft"), 243)]
    #[case(ProgramName("tknk"), 778)]
    fn subtower_weights_are_determined(
        subtower_weights: &HashMap<ProgramName, u32>,
        #[case] prog: ProgramName,
        #[case] exp_weight: u32,
    ) {
        assert_eq!(subtower_weights.get(&prog), Some(&exp_weight));
    }

    #[rstest]
    fn subtowers_are_iterated_in_order(relations: &HashMap<ProgramName, Relation>) {
        assert_equal(
            SubtowerIter::new(relations, ProgramName("tknk"))
                .take(3)
                .sorted(),
            [
                ProgramName("fwft"),
                ProgramName("padx"),
                ProgramName("ugml"),
            ],
        );
    }

    #[rstest]
    fn corrected_weight_is_determined(
        relations: &HashMap<ProgramName, Relation>,
        subtower_weights: &HashMap<ProgramName, u32>,
    ) {
        assert_matches!(correct_weight(relations, subtower_weights), Ok(60));
    }
}
