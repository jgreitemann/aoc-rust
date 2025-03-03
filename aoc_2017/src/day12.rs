use aoc_companion::prelude::*;

use std::collections::BTreeSet;

use itertools::Itertools;
use thiserror::Error;

pub(crate) struct Door {
    connections: Vec<Vec<usize>>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseError> {
        parse_input(input).map(|connections| Self { connections })
    }

    fn part1(&self) -> Result<usize, Error> {
        connected_component(0, &self.connections).map(|comp| comp.len())
    }

    fn part2(&self) -> Result<usize, Error> {
        components(&self.connections).map(|comps| comps.len())
    }
}

#[derive(Debug, Error)]
pub(crate) enum ParseError {
    #[error("A line in the input does not match the regex: '{line}'")]
    LineDoesNotMatchPattern { line: String },
    #[error("Index for line is out-of-order: expected {expected}, found {actual}")]
    IndexMismatch { expected: usize, actual: usize },
}

fn parse_input(input: &str) -> Result<Vec<Vec<usize>>, ParseError> {
    let re = regex::Regex::new(r"^(?P<index>\d+) <-> (?P<connections>\d+(?:, \d+)*)$").unwrap();
    input
        .lines()
        .map(|line| {
            re.captures(line)
                .ok_or_else(|| ParseError::LineDoesNotMatchPattern {
                    line: line.to_owned(),
                })
        })
        .enumerate()
        .map(|(expected, captures)| {
            let captures = captures?;
            let actual = captures["index"].parse().unwrap();
            if actual == expected {
                Ok(captures.name("connections").unwrap().as_str())
            } else {
                Err(ParseError::IndexMismatch { expected, actual })
            }
        })
        .map_ok(|connections| {
            connections
                .split(", ")
                .map(|s| s.parse().unwrap())
                .collect_vec()
        })
        .collect()
}

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("Index out-of-bounds: {0}")]
    IndexOutOfBounds(usize),
}

fn connected_component<C: AsRef<[usize]>>(
    seed: usize,
    connections: &[C],
) -> Result<BTreeSet<usize>, Error> {
    let mut to_check = vec![seed];
    let mut component = BTreeSet::new();
    while let Some(current) = to_check.pop() {
        if component.insert(current) {
            let connected = connections
                .get(current)
                .ok_or(Error::IndexOutOfBounds(current))?
                .as_ref();
            to_check.extend_from_slice(connected);
        }
    }

    Ok(component)
}

fn components<C: AsRef<[usize]>>(connections: &[C]) -> Result<BTreeSet<BTreeSet<usize>>, Error> {
    let mut components = BTreeSet::new();
    let mut indices: BTreeSet<usize> = (0..connections.len()).collect();

    while let Some(&index) = indices.iter().next() {
        let component = connected_component(index, connections)?;
        indices = indices.difference(&component).copied().collect();
        components.insert(component);
    }

    Ok(components)
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use itertools::assert_equal;

    use super::*;

    const EXAMPLE_INPUT: &str = r#"0 <-> 2
1 <-> 1
2 <-> 0, 3, 4
3 <-> 2, 4
4 <-> 2, 3, 6
5 <-> 6
6 <-> 4, 5"#;
    const EXAMPLE_CONNECTIONS: &[&[usize]] =
        &[&[2], &[1], &[0, 3, 4], &[2, 4], &[2, 3, 6], &[6], &[4, 5]];

    #[test]
    fn pipe_connections_are_parsed() {
        assert_eq!(parse_input(EXAMPLE_INPUT).unwrap(), EXAMPLE_CONNECTIONS);
    }

    #[test]
    fn connected_vertices_are_found() {
        assert_equal(
            connected_component(0, EXAMPLE_CONNECTIONS).unwrap(),
            [0, 2, 3, 4, 5, 6],
        );
        assert_equal(connected_component(1, EXAMPLE_CONNECTIONS).unwrap(), [1]);
        assert_matches!(
            connected_component(7, EXAMPLE_CONNECTIONS),
            Err(Error::IndexOutOfBounds(7))
        );
    }

    #[test]
    fn component_groups_are_found() {
        assert_equal(
            components(EXAMPLE_CONNECTIONS).unwrap(),
            [[0, 2, 3, 4, 5, 6].into(), [1].into()],
        );
    }
}
