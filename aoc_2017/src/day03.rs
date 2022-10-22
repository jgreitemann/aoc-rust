use std::{collections::HashMap, convert::Infallible, num::ParseIntError};

use aoc_companion::prelude::*;

pub struct Door {
    input: usize,
}

impl ParseInput for Door {
    type Error = ParseIntError;

    fn parse(input: &str) -> Result<Self, Self::Error> {
        Ok(Self {
            input: input.trim_end().parse()?,
        })
    }
}

impl Part1 for Door {
    type Output = i32;
    type Error = Infallible;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        Ok(manhattan_distance(
            SpiralIter::new().nth(self.input - 1).unwrap(),
        ))
    }
}

impl Part2 for Door {
    type Error = Infallible;
    type Output = i32;

    fn part2(&self) -> Result<Self::Output, Self::Error> {
        Ok(first_partial_sum_along_spiral_exceeding(self.input as i32))
    }
}

type Point = (i32, i32);

#[derive(Debug, Copy, Clone)]
enum Direction {
    East,
    North,
    West,
    South,
}

struct SpiralIter {
    current: Point,
    direction: Direction,
}

impl SpiralIter {
    fn new() -> Self {
        Self {
            current: (0, 0),
            direction: Direction::East,
        }
    }
}

impl Iterator for SpiralIter {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        let (x, y) = self.current;
        let next = match self.direction {
            Direction::East => (x + 1, y),
            Direction::North => (x, y + 1),
            Direction::West => (x - 1, y),
            Direction::South => (x, y - 1),
        };

        self.direction = match next {
            (x, y) if x == -y + 1 && y <= 0 => Direction::North,
            (x, y) if x == y && x > 0 => Direction::West,
            (x, y) if x == -y && y > 0 => Direction::South,
            (x, y) if x == y && x < 0 => Direction::East,
            _ => self.direction,
        };

        Some(std::mem::replace(&mut self.current, next))
    }
}

fn manhattan_distance(p: Point) -> i32 {
    p.0.abs() + p.1.abs()
}

fn neighbors(p: Point) -> impl Iterator<Item = Point> {
    [
        (1, 0),
        (1, 1),
        (0, 1),
        (-1, 1),
        (-1, 0),
        (-1, -1),
        (0, -1),
        (1, -1),
    ]
    .iter()
    .map(move |(x, y)| (x + p.0, y + p.1))
}

fn partial_sums_along_spiral() -> impl Iterator<Item = i32> {
    SpiralIter::new().scan(HashMap::new(), |map, p| {
        let sum = neighbors(p).flat_map(|q| map.get(&q)).sum();
        let next = std::cmp::max(1, sum);
        map.insert(p, next);
        Some(next)
    })
}

fn first_partial_sum_along_spiral_exceeding(threshold: i32) -> i32 {
    partial_sums_along_spiral()
        .skip_while(|&x| x <= threshold)
        .next()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::assert_equal;

    #[test]
    fn spiral_iterator_starts_at_origin() {
        assert_eq!(SpiralIter::new().next(), Some((0, 0)));
    }

    #[test]
    fn spiral_iterator_produces_the_first_12_points() {
        assert_equal(
            SpiralIter::new().take(12),
            [
                (0, 0),
                (1, 0),
                (1, 1),
                (0, 1),
                (-1, 1),
                (-1, 0),
                (-1, -1),
                (0, -1),
                (1, -1),
                (2, -1),
                (2, 0),
                (2, 1),
            ],
        );
    }

    #[test]
    fn manhattan_distance_is_correct_for_points_in_all_quadrants() {
        assert_eq!(manhattan_distance((0, 0)), 0);
        assert_eq!(manhattan_distance((2, 3)), 5);
        assert_eq!(manhattan_distance((-3, 4)), 7);
        assert_eq!(manhattan_distance((1, -8)), 9);
        assert_eq!(manhattan_distance((-2, -4)), 6);
    }

    #[test]
    fn manhattan_distance_is_correct_for_known_spiral_points() {
        assert_eq!(manhattan_distance(SpiralIter::new().nth(0).unwrap()), 0);
        assert_eq!(manhattan_distance(SpiralIter::new().nth(11).unwrap()), 3);
        assert_eq!(manhattan_distance(SpiralIter::new().nth(22).unwrap()), 2);
        assert_eq!(manhattan_distance(SpiralIter::new().nth(1023).unwrap()), 31);
    }

    #[test]
    fn neighbors_of_point() {
        assert_equal(
            neighbors((3, -2)),
            [
                (4, -2),
                (4, -1),
                (3, -1),
                (2, -1),
                (2, -2),
                (2, -3),
                (3, -3),
                (4, -3),
            ],
        );
    }

    #[test]
    fn partial_sums_along_spiral_for_first_12_points() {
        assert_equal(
            partial_sums_along_spiral().take(12),
            [1, 1, 2, 4, 5, 10, 11, 23, 25, 26, 54, 57],
        );
    }

    #[test]
    fn first_partial_sums_along_spiral_exceeding_certain_example_values() {
        assert_eq!(first_partial_sum_along_spiral_exceeding(10), 11);
        assert_eq!(first_partial_sum_along_spiral_exceeding(50), 54);
        assert_eq!(first_partial_sum_along_spiral_exceeding(100), 122);
        assert_eq!(first_partial_sum_along_spiral_exceeding(200), 304);
        assert_eq!(first_partial_sum_along_spiral_exceeding(500), 747);
    }
}
