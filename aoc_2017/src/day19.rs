use std::collections::HashMap;

use aoc_companion::prelude::*;
use aoc_utils::{geometry::Point, linalg::Vector};

pub(crate) struct Door {
    grid: Grid,
}

type Grid = HashMap<Vector<usize, 2>, u8>;

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Self {
        Self {
            grid: parse_grid(input),
        }
    }

    fn part1(&self) -> String {
        letters(walk_pipe(&self.grid))
    }

    fn part2(&self) -> usize {
        walk_pipe(&self.grid).count()
    }
}

fn parse_grid(input: &str) -> Grid {
    input
        .lines()
        .enumerate()
        .flat_map(|(row, line)| {
            line.as_bytes()
                .iter()
                .enumerate()
                .filter_map(move |(col, &b)| (b != b' ').then_some((Vector([col, row]), b)))
        })
        .collect()
}

fn find_start(grid: &Grid) -> Option<Vector<usize, 2>> {
    grid.keys().find(|v| matches!(v, Vector([_, 0]))).copied()
}

fn walk_pipe(grid: &Grid) -> impl Iterator<Item = u8> + use<'_> {
    std::iter::successors(
        find_start(grid).map(|pos| (pos, Vector([0, 1]))),
        |&(pos, dir)| {
            if grid[&pos] == b'+' {
                pos.nearest_neighbors().find_map(|next| {
                    let new_dir =
                        next.try_cast_as::<isize>().unwrap() - pos.try_cast_as::<isize>().unwrap();
                    (new_dir + dir != Vector::new() && grid.contains_key(&next))
                        .then_some((next, new_dir))
                })
            } else {
                let next = (pos.try_cast_as::<isize>().unwrap() + dir)
                    .try_cast_as::<usize>()
                    .ok()?;
                grid.contains_key(&next).then_some((next, dir))
            }
        },
    )
    .map(|(pos, _)| grid[&pos])
}

fn letters(pipe: impl Iterator<Item = u8>) -> String {
    String::from_utf8(pipe.filter(|c| c.is_ascii_alphabetic()).collect()).unwrap()
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    const EXAMPLE_INPUT: &str = "     |          
     |  +--+    
     A  |  C    
 F---|----E|--+ 
     |  |  |  D 
     +B-+  +--+ 
                ";

    const EXAMPLE_GRID_POINTS: &[(Vector<usize, 2>, u8)] = &[
        (Vector([5, 0]), b'|'),
        (Vector([5, 1]), b'|'),
        (Vector([5, 2]), b'A'),
        (Vector([5, 3]), b'|'),
        (Vector([5, 4]), b'|'),
        (Vector([5, 5]), b'+'),
        (Vector([6, 5]), b'B'),
        (Vector([7, 5]), b'-'),
        (Vector([8, 5]), b'+'),
        (Vector([8, 4]), b'|'),
        (Vector([8, 3]), b'-'),
        (Vector([8, 2]), b'|'),
        (Vector([8, 1]), b'+'),
        (Vector([9, 1]), b'-'),
        (Vector([10, 1]), b'-'),
        (Vector([11, 1]), b'+'),
        (Vector([11, 2]), b'C'),
        (Vector([11, 3]), b'|'),
        (Vector([11, 4]), b'|'),
        (Vector([11, 5]), b'+'),
        (Vector([12, 5]), b'-'),
        (Vector([13, 5]), b'-'),
        (Vector([14, 5]), b'+'),
        (Vector([14, 4]), b'D'),
        (Vector([14, 3]), b'+'),
        (Vector([13, 3]), b'-'),
        (Vector([12, 3]), b'-'),
        (Vector([10, 3]), b'E'),
        (Vector([9, 3]), b'-'),
        (Vector([7, 3]), b'-'),
        (Vector([6, 3]), b'-'),
        (Vector([4, 3]), b'-'),
        (Vector([3, 3]), b'-'),
        (Vector([2, 3]), b'-'),
        (Vector([1, 3]), b'F'),
    ];

    const EXAMPLE_PIPE_CHARS: &[u8] = b"||A||+B-+|-|+--+C||+--+D+--|E----|---F";

    #[test]
    fn parse_example_grid() {
        assert_eq!(
            parse_grid(EXAMPLE_INPUT),
            Grid::from_iter(EXAMPLE_GRID_POINTS.iter().copied()),
        )
    }

    #[test]
    fn example_start_point() {
        assert_eq!(
            find_start(&Grid::from_iter(EXAMPLE_GRID_POINTS.iter().copied())),
            Some(Vector([5, 0]))
        )
    }

    #[test]
    fn example_pipe_walk() {
        assert_eq!(
            walk_pipe(&Grid::from_iter(EXAMPLE_GRID_POINTS.iter().copied())).collect_vec(),
            EXAMPLE_PIPE_CHARS
        );
    }

    #[test]
    fn example_string() {
        assert_eq!(letters(EXAMPLE_PIPE_CHARS.iter().copied()), "ABCDEF");
    }

    #[test]
    fn example_distance() {
        assert_eq!(EXAMPLE_PIPE_CHARS.len(), 38);
    }
}
