use std::{
    collections::{HashMap, HashSet},
    num::ParseIntError,
};

use aoc_companion::prelude::*;
use aoc_utils::{
    geometry::Point,
    linalg::{ParseVectorError, Vector},
};
use itertools::Itertools;

pub(crate) struct Door {
    bytes: Vec<Vector<usize, 2>>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        Ok(Door {
            bytes: parse_bytes(input)?,
        })
    }

    fn part1(&self) -> usize {
        find_shortest_path_with_fallen_bytes(&self.bytes[..1024], [71, 71])
    }

    fn part2(&self) -> String {
        first_byte_to_cut_off_exit(&self.bytes, [71, 71])
            .unwrap()
            .iter()
            .join(",")
    }
}

fn parse_bytes(input: &str) -> Result<Vec<Vector<usize, 2>>, ParseVectorError<ParseIntError>> {
    input.lines().map(str::parse).try_collect()
}

fn find_shortest_path_with_fallen_bytes(bytes: &[Vector<usize, 2>], shape: [usize; 2]) -> usize {
    let fallen_bytes: HashSet<Vector<usize, 2>> = bytes.iter().cloned().collect();
    let mut distances = ndarray::Array2::from_elem(shape, usize::MAX);
    distances[Vector::default()] = 0;

    let mut todo = HashSet::from([Vector::default()]);
    while let Some(current) = todo.iter().next().cloned() {
        todo.remove(&current);
        let current_dist = distances[current] + 1;
        todo.extend(
            current
                .nearest_neighbors()
                .filter(|n| !fallen_bytes.contains(n))
                .filter(|n| {
                    let better = distances.get(*n).is_some_and(|d| current_dist < *d);
                    if better {
                        distances[*n] = current_dist;
                    }
                    better
                }),
        );
    }

    distances[shape.map(|c| c - 1)]
}

fn first_byte_to_cut_off_exit(
    bytes: &[Vector<usize, 2>],
    shape: [usize; 2],
) -> Option<Vector<usize, 2>> {
    let mut cache = HashMap::new();
    let ends: Vec<_> = (0..bytes.len()).collect();

    ends.binary_search_by_key(&false, |&end| {
        *cache.entry(end).or_insert_with(|| {
            find_shortest_path_with_fallen_bytes(&bytes[..end], shape) == usize::MAX
        })
    })
    .ok()
    .map(|end| bytes[end])
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "\
5,4
4,2
4,5
3,0
2,1
6,3
2,4
1,5
0,6
3,3
2,6
5,1
1,2
5,5
2,5
6,5
1,4
0,4
6,4
1,1
6,1
1,0
0,5
1,6
2,0";

    #[test]
    fn shortest_path_with_fallen_bytes_in_example() {
        let bytes = parse_bytes(EXAMPLE_INPUT).unwrap();
        assert_eq!(
            find_shortest_path_with_fallen_bytes(&bytes[0..12], [7, 7]),
            22
        )
    }

    #[test]
    fn first_byte_to_cut_off_exit_in_example() {
        let bytes = parse_bytes(EXAMPLE_INPUT).unwrap();
        assert_eq!(
            first_byte_to_cut_off_exit(&bytes, [7, 7]),
            Some(Vector([6, 1]))
        )
    }
}
