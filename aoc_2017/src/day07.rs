use aoc_companion::prelude::*;
use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

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
}

pub struct Door<'input> {
    forward_relations: HashMap<ProgramName<'input>, ForwardRelation<'input>>,
}

impl<'input> ParseInput<'input> for Door<'input> {
    type Error = ParseError;

    fn parse(input: &'input str) -> Result<Self, Self::Error> {
        let forward_relations = parse_input(input)?;
        Ok(Self { forward_relations })
    }
}

impl<'input> Part1 for Door<'input> {
    type Output = ProgramName<'input>;
    type Error = GraphError;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        find_bottom_program(&self.forward_relations)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ProgramName<'input>(&'input str);

impl Display for ProgramName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

#[derive(Debug)]
struct ForwardRelation<'input> {
    weight: u32,
    decendents: HashSet<ProgramName<'input>>,
}

fn parse_input<'input>(
    input: &'input str,
) -> Result<HashMap<ProgramName<'input>, ForwardRelation<'input>>, ParseError> {
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
                ForwardRelation {
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
    forward_relations: &HashMap<ProgramName<'input>, ForwardRelation<'input>>,
) -> Result<ProgramName<'input>, GraphError> {
    let dependent_programs: HashSet<ProgramName> = forward_relations
        .values()
        .flat_map(|r| r.decendents.iter())
        .cloned()
        .collect();
    forward_relations
        .keys()
        .filter(|k| !dependent_programs.contains(k))
        .cloned()
        .at_most_one()
        .map_err(|_| GraphError::MultipleTreeComponents)?
        .ok_or(GraphError::GraphIsCyclic)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
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
    fn forward_relations() -> HashMap<ProgramName<'static>, ForwardRelation<'static>> {
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
        forward_relations: &HashMap<ProgramName, ForwardRelation>,
        #[case] prog: &str,
        #[case] exp_weight: u32,
        #[case] names: &[&str],
    ) {
        let matches = |d: &HashSet<ProgramName>, names: &[&str]| {
            names.len() == d.len() && names.into_iter().all(|name| d.contains(&ProgramName(name)))
        };
        assert_matches!(
                forward_relations.get(&ProgramName(prog)),
                Some(ForwardRelation{ weight, decendents })
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
    fn bottom_program_is_found(forward_relations: &HashMap<ProgramName, ForwardRelation>) {
        assert_matches!(
            find_bottom_program(forward_relations),
            Ok(ProgramName("tknk"))
        );
    }

    #[test]
    fn bottom_program_does_not_exist_because_of_cycles() {
        let forward_relations = parse_input(
            r#"A (42) -> B
B (42) -> C
C (42) -> A
"#,
        )
        .unwrap();
        assert_matches!(
            find_bottom_program(&forward_relations),
            Err(GraphError::GraphIsCyclic)
        );
    }

    #[test]
    fn bottom_program_is_ambiguous_because_graph_has_multiple_components() {
        let forward_relations = parse_input(
            r#"A (42) -> B
B (42) -> C
D (42) -> E
"#,
        )
        .unwrap();
        assert_matches!(
            find_bottom_program(&forward_relations),
            Err(GraphError::MultipleTreeComponents)
        );
    }
}
