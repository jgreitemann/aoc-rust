use std::collections::HashSet;

use aoc_companion::prelude::*;
use aoc_utils::{
    geometry::{map_bounds, Point},
    linalg::Vector,
};
use itertools::Itertools;

pub(crate) struct Door {
    map: ndarray::Array2<u8>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Door> {
        Ok(Door {
            map: parse_map(input)?,
        })
    }

    fn part1(&self) -> usize {
        trailhead_scores(&self.map).sum()
    }

    fn part2(&self) -> usize {
        trailhead_ratings(&self.map).sum()
    }
}

fn parse_map(input: &str) -> Result<ndarray::Array2<u8>, ndarray::ShapeError> {
    let data = input
        .lines()
        .flat_map(|line| line.as_bytes().iter().map(|b| b - b'0'))
        .collect();
    let bounds = map_bounds(input);
    ndarray::Array2::from_shape_vec(bounds.map(|b| b.end as usize), data)
}

fn reachable_niners<Coll>(start: Vector<usize, 2>, map: &ndarray::Array2<u8>) -> Coll
where
    Coll: FromIterator<Vector<usize, 2>>,
    for<'a> &'a Coll: IntoIterator<Item = &'a Vector<usize, 2>>,
{
    ((map[start] + 1)..=9).fold(std::iter::once(start).collect(), |points, level| {
        points
            .into_iter()
            .flat_map(|p| p.nearest_neighbors())
            .filter(|p| map.get(*p) == Some(&level))
            .collect()
    })
}

fn trailhead_iter(map: &ndarray::Array2<u8>) -> impl Iterator<Item = Vector<usize, 2>> + use<'_> {
    Itertools::cartesian_product(0..map.dim().0, 0..map.dim().1)
        .map(|(x, y)| Vector([x, y]))
        .filter(|v| map.get(*v) == Some(&0))
}

fn trailhead_scores(map: &ndarray::Array2<u8>) -> impl Iterator<Item = usize> + use<'_> {
    trailhead_iter(map).map(|v| reachable_niners::<HashSet<Vector<usize, 2>>>(v, map).len())
}

fn trailhead_ratings(map: &ndarray::Array2<u8>) -> impl Iterator<Item = usize> + use<'_> {
    trailhead_iter(map).map(|v| reachable_niners::<Vec<Vector<usize, 2>>>(v, map).len())
}

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;

    use super::*;

    const EXAMPLE_INPUT: &str = "\
89010123
78121874
87430965
96549874
45678903
32019012
01329801
10456732";

    lazy_static! {
        static ref EXAMPLE_MAP: ndarray::Array2<u8> = ndarray::array![
            [8, 9, 0, 1, 0, 1, 2, 3],
            [7, 8, 1, 2, 1, 8, 7, 4],
            [8, 7, 4, 3, 0, 9, 6, 5],
            [9, 6, 5, 4, 9, 8, 7, 4],
            [4, 5, 6, 7, 8, 9, 0, 3],
            [3, 2, 0, 1, 9, 0, 1, 2],
            [0, 1, 3, 2, 9, 8, 0, 1],
            [1, 0, 4, 5, 6, 7, 3, 2],
        ];
    }

    #[test]
    fn parse_example_map() {
        assert_eq!(parse_map(EXAMPLE_INPUT).unwrap(), *EXAMPLE_MAP);
    }

    #[test]
    fn example_trailhead_scores() {
        assert_eq!(
            trailhead_scores(&EXAMPLE_MAP).collect_vec(),
            [5, 6, 5, 3, 1, 3, 5, 3, 5]
        );
    }

    #[test]
    fn example_trailhead_ratings() {
        assert_eq!(
            trailhead_ratings(&EXAMPLE_MAP).collect_vec(),
            [20, 24, 10, 4, 1, 4, 5, 8, 5]
        );
    }
}
