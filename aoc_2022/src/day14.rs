use aoc_companion::prelude::*;
use aoc_utils::linalg::Vector;

use itertools::Itertools;
use thiserror::Error;

use std::collections::HashSet;
use std::str::FromStr;

pub struct Door {
    paths: Vec<Path>,
}

impl ParseInput<'_> for Door {
    type Error = ParseError;

    fn parse(input: &str) -> Result<Self, Self::Error> {
        parse_input(input).map(|paths| Self { paths })
    }
}

impl Part1 for Door {
    type Output = usize;
    type Error = std::convert::Infallible;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        let mut pit = Pit::new_bottomless(&self.paths);
        pit.fill_up();
        Ok(pit.settled_sand.len())
    }
}

impl Part2 for Door {
    type Output = usize;
    type Error = std::convert::Infallible;

    fn part2(&self) -> Result<Self::Output, Self::Error> {
        let mut pit = Pit::new_with_floor(&self.paths);
        pit.fill_up();
        Ok(pit.settled_sand.len())
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("A set of coordinates is not comma-separated.")]
    CoordinatesNotSeparatedByComma,
    #[error(transparent)]
    ParseCoordinate(#[from] std::num::ParseIntError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Path(Vec<Vector<i32, 2>>);

impl Path {
    fn contains(&self, q: &Vector<i32, 2>) -> bool {
        self.0.windows(2).any(|window| {
            let &[p1, p2] = window else {
                panic!("Window size should match length of slice destructuring")
            };
            (q[0] == p1[0]
                && q[0] == p2[0]
                && (p1[1].min(p2[1])..=p1[1].max(p2[1])).contains(&q[1]))
                || (q[1] == p1[1]
                    && q[1] == p2[1]
                    && (p1[0].min(p2[0])..=p1[0].max(p2[0])).contains(&q[0]))
        })
    }
}

impl FromStr for Path {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split(" -> ")
            .map(|coords| {
                coords
                    .split_once(',')
                    .ok_or(ParseError::CoordinatesNotSeparatedByComma)
            })
            .map(|res| res.and_then(|(x_str, y_str)| Ok(Vector([x_str.parse()?, y_str.parse()?]))))
            .try_collect()
            .map(Path)
    }
}

fn parse_input(input: &str) -> Result<Vec<Path>, ParseError> {
    input.lines().map(str::parse).try_collect()
}

const SOURCE: Vector<i32, 2> = Vector([500, 0]);
const STRAIGHT_DOWN: Vector<i32, 2> = Vector([0, 1]);
const LEFT_DOWN: Vector<i32, 2> = Vector([-1, 1]);
const RIGHT_DOWN: Vector<i32, 2> = Vector([1, 1]);

#[derive(Debug)]
struct Pit {
    falling_sand_stack: Vec<Vector<i32, 2>>,
    settled_sand: HashSet<Vector<i32, 2>>,
    paths: Vec<Path>,
    floor: i32,
}

impl Pit {
    fn new_bottomless(paths: &[Path]) -> Self {
        let floor = paths
            .iter()
            .map(|p| p.0.iter().map(|q| q[1]).max().unwrap_or(0))
            .max()
            .unwrap_or(0);
        Pit {
            falling_sand_stack: vec![SOURCE],
            settled_sand: HashSet::new(),
            paths: paths.to_vec(),
            floor,
        }
    }

    fn new_with_floor(paths: &[Path]) -> Self {
        let floor = paths
            .iter()
            .map(|p| p.0.iter().map(|q| q[1]).max().unwrap_or(0))
            .max()
            .unwrap_or(0)
            + 2;
        let paths = paths
            .iter()
            .chain(
                [Path(vec![
                    Vector([i32::MIN, floor]),
                    Vector([i32::MAX, floor]),
                ])]
                .iter(),
            )
            .cloned()
            .collect();
        Pit {
            falling_sand_stack: vec![SOURCE],
            settled_sand: HashSet::new(),
            paths,
            floor,
        }
    }

    fn is_obstructed(&self, q: &Vector<i32, 2>) -> bool {
        self.settled_sand.contains(q) || self.paths.iter().any(|p| p.contains(q))
    }

    fn try_point(&self, q: Vector<i32, 2>) -> Option<Vector<i32, 2>> {
        Some(q).filter(|q| !self.is_obstructed(q))
    }

    fn fill_one(&mut self) -> bool {
        if let Some(mut falling_sand) = self.falling_sand_stack.pop() {
            while let Some(q) = self
                .try_point(falling_sand + STRAIGHT_DOWN)
                .or_else(|| self.try_point(falling_sand + LEFT_DOWN))
                .or_else(|| self.try_point(falling_sand + RIGHT_DOWN))
            {
                self.falling_sand_stack.push(falling_sand);
                falling_sand = q;
                if falling_sand[1] > self.floor {
                    return false;
                }
            }

            self.settled_sand.insert(falling_sand);
            return true;
        }

        false
    }

    fn fill_up(&mut self) {
        while self.fill_one() {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "498,4 -> 498,6 -> 496,6\n\
                                 503,4 -> 502,4 -> 502,9 -> 494,9";

    //const EXAMPLE_PATHS: &[&[Vector<i32, 2>]] = &[
    fn example_paths() -> Vec<Path> {
        vec![
            Path(vec![Vector([498, 4]), Vector([498, 6]), Vector([496, 6])]),
            Path(vec![
                Vector([503, 4]),
                Vector([502, 4]),
                Vector([502, 9]),
                Vector([494, 9]),
            ]),
        ]
    }

    #[test]
    fn input_is_parsed() {
        assert_eq!(parse_input(EXAMPLE_INPUT).unwrap(), example_paths());
    }

    #[test]
    fn pit_floor_is_determined() {
        let paths = example_paths();
        let pit = Pit::new_bottomless(&paths);
        assert_eq!(pit.floor, 9);
    }

    #[test]
    fn settled_sand_at_equilibrium() {
        let paths = example_paths();
        let mut pit = Pit::new_bottomless(&paths);
        pit.fill_up();
        assert_eq!(pit.settled_sand.len(), 24);
    }

    #[test]
    fn settled_sand_until_source_is_blocked() {
        let paths = example_paths();
        let mut pit = Pit::new_with_floor(&paths);
        pit.fill_up();
        assert_eq!(pit.settled_sand.len(), 93);
    }
}
