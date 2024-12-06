use std::{cell::Cell, collections::HashSet, ops::Range};

use aoc_companion::prelude::*;
use aoc_utils::linalg::Vector;
use itertools::Itertools;
use rayon::prelude::*;
use tap::Tap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Door {
    map: Map,
    starting_guard: Guard,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Map {
    obstacles: HashSet<Vector<i32, 2>>,
    bounds: (Range<i32>, Range<i32>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Guard {
    pos: Vector<i32, 2>,
    heading: Vector<i32, 2>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ParseError {
    #[error("Encountered an unrecognized character when parsing input: {0}")]
    UnrecognizedCharError(u8),
    #[error("No starting position marked on map")]
    NoStartingPosition,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseError> {
        let rows = input.lines().count() as i32;
        let cols = input.lines().next().map(str::len).unwrap_or(0) as i32;

        let starting_pos: Cell<Option<Vector<i32, 2>>> = Cell::default();

        let obstacles = input
            .lines()
            .enumerate()
            .flat_map(|(row, line)| {
                let cell = &starting_pos;
                line.as_bytes()
                    .iter()
                    .enumerate()
                    .filter_map(move |(col, byte)| match *byte {
                        b'.' => None,
                        b'#' => Some(Ok(Vector([col, row]).try_cast_as().unwrap())),
                        b'^' => {
                            cell.set(Vector([col, row]).try_cast_as().ok());
                            None
                        }
                        other => Some(Err(ParseError::UnrecognizedCharError(other))),
                    })
            })
            .try_collect()?;

        Ok(Door {
            map: Map {
                obstacles,
                bounds: (0..cols, 0..rows),
            },
            starting_guard: Guard {
                pos: starting_pos.take().ok_or(ParseError::NoStartingPosition)?,
                heading: Vector([0, -1]),
            },
        })
    }

    fn part1(&self) -> usize {
        count_unique_positions(self.path())
    }

    fn part2(&self) -> usize {
        self.count_incursions_resulting_in_loop()
    }
}

impl Map {
    fn is_free(&self, pos: Vector<i32, 2>) -> bool {
        !self.obstacles.contains(&pos)
    }

    fn is_in_bounds(&self, pos: Vector<i32, 2>) -> bool {
        self.bounds.0.contains(&pos.0[0]) && self.bounds.1.contains(&pos.0[1])
    }
}

fn step(mut current: Guard, map: &Map) -> Guard {
    loop {
        let next_pos = current.pos + current.heading;
        if map.is_free(next_pos) {
            break Guard {
                pos: next_pos,
                heading: current.heading,
            };
        } else {
            current.heading = Vector([-current.heading.0[1], current.heading.0[0]]);
        }
    }
}

impl Door {
    fn path(&self) -> impl Iterator<Item = Guard> + use<'_> {
        itertools::iterate(self.starting_guard, |&current| step(current, &self.map))
            .take_while(|guard| self.map.is_in_bounds(guard.pos))
    }

    fn count_incursions_resulting_in_loop(&self) -> usize {
        let (col_bounds, row_bounds) = self.map.bounds.clone();
        Itertools::cartesian_product(col_bounds, row_bounds)
            .par_bridge()
            .map(|(col, row)| Vector([col, row]))
            .filter(|p| self.starting_guard.pos != *p)
            .filter(|p| !self.map.obstacles.contains(p))
            .map(|p| {
                self.clone().tap_mut(|clone| {
                    clone.map.obstacles.insert(p);
                })
            })
            .filter(|door| is_loop(door.path()))
            .count()
    }
}

fn count_unique_positions(path: impl Iterator<Item = Guard>) -> usize {
    path.map(|g| g.pos).unique().count()
}

fn is_loop(path: impl Iterator<Item = Guard>) -> bool {
    path.duplicates().next().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "\
....#.....
.........#
..........
..#.......
.......#..
..........
.#..^.....
........#.
#.........
......#...";

    fn example_door() -> Door {
        Door {
            map: Map {
                obstacles: HashSet::from([
                    Vector([4, 0]),
                    Vector([9, 1]),
                    Vector([2, 3]),
                    Vector([7, 4]),
                    Vector([1, 6]),
                    Vector([8, 7]),
                    Vector([0, 8]),
                    Vector([6, 9]),
                ]),
                bounds: (0..10, 0..10),
            },
            starting_guard: Guard {
                pos: Vector([4, 6]),
                heading: Vector([0, -1]),
            },
        }
    }

    #[test]
    fn example_input_can_be_parsed() {
        assert_eq!(Door::parse(EXAMPLE_INPUT).unwrap(), example_door());
    }

    #[test]
    fn unique_positions_in_example_path() {
        assert_eq!(count_unique_positions(example_door().path()), 41);
    }

    #[test]
    fn possible_incursions_into_time() {
        assert_eq!(example_door().count_incursions_resulting_in_loop(), 6);
    }
}
