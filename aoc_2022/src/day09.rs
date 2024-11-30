use aoc_companion::prelude::*;
use aoc_utils::{geometry::Point, linalg::*};

use thiserror::Error;

use std::str::FromStr;

pub(crate) struct Door {
    motions: Vec<Motion>,
}

impl<'input> ParseInput<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseError> {
        parse_input(input).map(|motions| Self { motions })
    }
}

impl Part1 for Door {
    fn part1(&self) -> usize {
        let directions: Vec<_> = as_directions(self.motions.iter().cloned()).collect();
        count_unique_positions(directions.into_iter().head_positions().tail_positions())
    }
}

impl Part2 for Door {
    fn part2(&self) -> usize {
        let directions: Vec<_> = as_directions(self.motions.iter().cloned()).collect();
        count_unique_positions(directions.into_iter().head_positions().tie_knots(10))
    }
}

#[derive(Debug, Error)]
pub(crate) enum ParseError {
    #[error("Missing space in motion line")]
    NoSpace,
    #[error("Motion direction must be either U, D, L, or R; found: {0:?}")]
    InvalidDirection(String),
    #[error("Motion length is not a number: {0}")]
    InvalidMotionLength(#[from] std::num::ParseIntError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Motion {
    direction: Direction,
    length: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn as_vector(&self) -> Vector<i32, 2> {
        match self {
            Direction::Up => Vector([0, 1]),
            Direction::Down => Vector([0, -1]),
            Direction::Left => Vector([-1, 0]),
            Direction::Right => Vector([1, 0]),
        }
    }
}

impl FromStr for Direction {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "U" => Ok(Direction::Up),
            "D" => Ok(Direction::Down),
            "L" => Ok(Direction::Left),
            "R" => Ok(Direction::Right),
            _ => Err(ParseError::InvalidDirection(s.to_owned())),
        }
    }
}

impl FromStr for Motion {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (dir_str, length_str) = s.split_once(' ').ok_or(ParseError::NoSpace)?;
        Ok(Motion {
            direction: dir_str.parse()?,
            length: length_str.parse()?,
        })
    }
}

fn parse_input(input: &str) -> Result<Vec<Motion>, ParseError> {
    input.lines().map(str::parse).collect()
}

struct HeadPositions<I>
where
    I: Iterator<Item = Direction>,
{
    directions: I,
    pos: Option<Vector<i32, 2>>,
}

impl<I> Iterator for HeadPositions<I>
where
    I: Iterator<Item = Direction>,
{
    type Item = Vector<i32, 2>;

    fn next(&mut self) -> Option<Self::Item> {
        let new_pos = self
            .pos
            .and_then(|p| self.directions.next().map(|m| p + m.as_vector()));
        std::mem::replace(&mut self.pos, new_pos)
    }
}

fn as_directions(motions: impl Iterator<Item = Motion>) -> impl Iterator<Item = Direction> {
    motions
        .flat_map(|Motion { direction, length }| std::iter::repeat(direction).take(length as usize))
}

#[derive(Debug, Clone)]
struct TailsPositions<I>
where
    I: Iterator<Item = Vector<i32, 2>>,
{
    head_iter: I,
    pos: Option<Vector<i32, 2>>,
}

impl<I> Iterator for TailsPositions<I>
where
    I: Iterator<Item = Vector<i32, 2>>,
{
    type Item = Vector<i32, 2>;
    fn next(&mut self) -> Option<Self::Item> {
        let next_head = self.head_iter.next();
        if let Some(p) = self.pos {
            self.pos = next_head.map(|h| {
                if (h - p).norm_l2() > 1.5 {
                    p.neighbors().min_by_key(|&v| (h - v).norm_l2_sq()).unwrap()
                } else {
                    p
                }
            });
        } else {
            self.pos = next_head;
        }
        self.pos
    }
}

fn count_unique_positions(iter: impl Iterator<Item = Vector<i32, 2>>) -> usize {
    iter.collect::<std::collections::HashSet<_>>().len()
}

trait PosIterator: Iterator<Item = Vector<i32, 2>> + Sized + 'static {
    fn tail_positions(self) -> Box<dyn Iterator<Item = Vector<i32, 2>>>;
    fn tie_knots(self, count: usize) -> Box<dyn Iterator<Item = Vector<i32, 2>>> {
        if count > 1 {
            self.tail_positions().tie_knots(count - 1)
        } else {
            Box::new(self)
        }
    }
}

impl<T> PosIterator for T
where
    T: Iterator<Item = Vector<i32, 2>> + Sized + 'static,
{
    fn tail_positions(self) -> Box<dyn Iterator<Item = Vector<i32, 2>>> {
        Box::new(TailsPositions {
            head_iter: self,
            pos: None,
        })
    }
}

trait DirIterator: Iterator<Item = Direction> + Sized {
    fn head_positions(self) -> HeadPositions<Self>;
}

impl<T> DirIterator for T
where
    T: Iterator<Item = Direction> + Sized,
{
    fn head_positions(self) -> HeadPositions<Self> {
        HeadPositions {
            directions: self,
            pos: Some(Vector::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use itertools::assert_equal;

    const EXAMPLE_INPUT: &str = r"R 4
U 4
L 3
D 1
R 4
D 1
L 5
R 2";

    const LARGE_INPUT: &str = r"R 5
U 8
L 8
D 3
R 17
D 10
L 25
U 20";

    const EXAMPLE_MOTIONS: &[Motion] = &[
        Motion {
            direction: Direction::Right,
            length: 4,
        },
        Motion {
            direction: Direction::Up,
            length: 4,
        },
        Motion {
            direction: Direction::Left,
            length: 3,
        },
        Motion {
            direction: Direction::Down,
            length: 1,
        },
        Motion {
            direction: Direction::Right,
            length: 4,
        },
        Motion {
            direction: Direction::Down,
            length: 1,
        },
        Motion {
            direction: Direction::Left,
            length: 5,
        },
        Motion {
            direction: Direction::Right,
            length: 2,
        },
    ];

    const EXAMPLE_HEAD_POSITIONS: &[Vector<i32, 2>] = &[
        Vector([0, 0]),
        Vector([1, 0]),
        Vector([2, 0]),
        Vector([3, 0]),
        Vector([4, 0]),
        Vector([4, 1]),
        Vector([4, 2]),
        Vector([4, 3]),
        Vector([4, 4]),
        Vector([3, 4]),
        Vector([2, 4]),
        Vector([1, 4]),
        Vector([1, 3]),
        Vector([2, 3]),
        Vector([3, 3]),
        Vector([4, 3]),
        Vector([5, 3]),
        Vector([5, 2]),
        Vector([4, 2]),
        Vector([3, 2]),
        Vector([2, 2]),
        Vector([1, 2]),
        Vector([0, 2]),
        Vector([1, 2]),
        Vector([2, 2]),
    ];

    const EXAMPLE_TAIL_POSITIONS: &[Vector<i32, 2>] = &[
        Vector([0, 0]),
        Vector([0, 0]),
        Vector([1, 0]),
        Vector([2, 0]),
        Vector([3, 0]),
        Vector([3, 0]),
        Vector([4, 1]),
        Vector([4, 2]),
        Vector([4, 3]),
        Vector([4, 3]),
        Vector([3, 4]),
        Vector([2, 4]),
        Vector([2, 4]),
        Vector([2, 4]),
        Vector([2, 4]),
        Vector([3, 3]),
        Vector([4, 3]),
        Vector([4, 3]),
        Vector([4, 3]),
        Vector([4, 3]),
        Vector([3, 2]),
        Vector([2, 2]),
        Vector([1, 2]),
        Vector([1, 2]),
        Vector([1, 2]),
    ];

    #[test]
    fn input_is_parsed() {
        assert_eq!(&parse_input(EXAMPLE_INPUT).unwrap(), EXAMPLE_MOTIONS);
    }

    #[test]
    fn head_positions() {
        assert_equal(
            as_directions(EXAMPLE_MOTIONS.iter().cloned()).head_positions(),
            EXAMPLE_HEAD_POSITIONS.iter().cloned(),
        );
    }

    #[test]
    fn tail_positions() {
        assert_equal(
            EXAMPLE_HEAD_POSITIONS.iter().cloned().tail_positions(),
            EXAMPLE_TAIL_POSITIONS.iter().cloned(),
        );
    }

    #[test]
    fn number_of_unique_tail_positions() {
        assert_eq!(
            count_unique_positions(EXAMPLE_TAIL_POSITIONS.iter().cloned()),
            13
        );
    }

    #[test]
    fn ten_fold_knot_unique_tail_positions() {
        assert_eq!(
            count_unique_positions(
                as_directions(parse_input(LARGE_INPUT).unwrap().into_iter())
                    .head_positions()
                    .tie_knots(10)
            ),
            36
        );
    }
}
