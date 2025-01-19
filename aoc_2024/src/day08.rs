use std::{
    collections::{HashMap, HashSet},
    ops::Range,
};

use aoc_companion::prelude::*;
use aoc_utils::{geometry::try_map_bounds, linalg::Vector};
use itertools::Itertools;

pub(crate) struct Door {
    bounds: [Range<i32>; 2],
    antennae: HashMap<u8, Vec<Vector<i32, 2>>>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Self {
        Door {
            bounds: try_map_bounds(input).unwrap(),
            antennae: parse_antennae(input),
        }
    }

    fn part1(&self) -> usize {
        self.unique_antennae_in_bounds([1]).len()
    }
    fn part2(&self) -> usize {
        self.unique_antennae_in_bounds(0..).len()
    }
}

fn parse_antennae(input: &str) -> HashMap<u8, Vec<Vector<i32, 2>>> {
    input
        .lines()
        .enumerate()
        .flat_map(|(row, line)| {
            line.bytes().enumerate().filter_map(move |(col, byte)| {
                (byte != b'.').then_some((byte, Vector([col as i32, row as i32])))
            })
        })
        .into_group_map()
}

fn antinodes<O: IntoIterator<Item = usize>>(
    first: Vector<i32, 2>,
    second: Vector<i32, 2>,
    bounds: &[Range<i32>; 2],
    orders: O,
) -> impl IntoIterator<Item = Vector<i32, 2>> + use<'_, O> {
    let delta = second - first;
    orders
        .into_iter()
        .map(move |order| second + delta * order as i32)
        .take_while(|n| n.in_bounds(bounds))
}

impl Door {
    fn unique_antennae_in_bounds(
        &self,
        orders: impl IntoIterator<Item = usize> + Clone,
    ) -> HashSet<Vector<i32, 2>> {
        self.antennae
            .values()
            .flat_map(|antennae| {
                antennae
                    .iter()
                    .tuple_combinations()
                    .flat_map(|(a, b)| [(a, b), (b, a)])
                    .flat_map(|(&a, &b)| antinodes(a, b, &self.bounds, orders.clone()))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use super::*;

    const EXAMPLE_INPUT: &str = "\
............
........0...
.....0......
.......0....
....0.......
......A.....
............
............
........A...
.........A..
............
............";

    const EXAMPLE_BOUNDS: [Range<i32>; 2] = [0..12, 0..12];

    const EXAMPLE_1ST_ORDER_ANTINODES: &[Vector<i32, 2>] = &[
        Vector([0, 7]),
        Vector([1, 5]),
        Vector([2, 3]),
        Vector([3, 1]),
        Vector([3, 6]),
        Vector([4, 2]),
        Vector([6, 0]),
        Vector([6, 5]),
        Vector([7, 7]),
        Vector([9, 4]),
        Vector([10, 2]),
        Vector([10, 10]),
        Vector([10, 11]),
        Vector([11, 0]),
    ];

    static EXAMPLE_ANTENNAE: LazyLock<HashMap<u8, Vec<Vector<i32, 2>>>> = LazyLock::new(|| {
        HashMap::from([
            (
                b'0',
                vec![
                    Vector([8, 1]),
                    Vector([5, 2]),
                    Vector([7, 3]),
                    Vector([4, 4]),
                ],
            ),
            (b'A', vec![Vector([6, 5]), Vector([8, 8]), Vector([9, 9])]),
        ])
    });

    #[test]
    fn parse_example_bounds() {
        assert_eq!(try_map_bounds(EXAMPLE_INPUT).unwrap(), EXAMPLE_BOUNDS);
    }

    #[test]
    fn parse_example_antennae() {
        assert_eq!(parse_antennae(EXAMPLE_INPUT), *EXAMPLE_ANTENNAE);
    }

    #[test]
    fn first_order_antinodes_of_antenna_pair() {
        assert_eq!(
            antinodes(Vector([4, 3]), Vector([5, 5]), &EXAMPLE_BOUNDS, [1])
                .into_iter()
                .exactly_one()
                .ok()
                .unwrap(),
            Vector([6, 7]),
        );
        assert_eq!(
            antinodes(Vector([5, 5]), Vector([4, 3]), &EXAMPLE_BOUNDS, [1])
                .into_iter()
                .exactly_one()
                .ok()
                .unwrap(),
            Vector([3, 1]),
        );
    }

    #[test]
    fn find_first_order_antinodes_in_bounds() {
        assert_eq!(
            Door {
                bounds: EXAMPLE_BOUNDS,
                antennae: EXAMPLE_ANTENNAE.clone()
            }
            .unique_antennae_in_bounds([1]),
            HashSet::from_iter(EXAMPLE_1ST_ORDER_ANTINODES.iter().copied())
        );
    }

    #[test]
    fn count_all_antinodes_in_bounds() {
        assert_eq!(
            Door {
                bounds: EXAMPLE_BOUNDS,
                antennae: EXAMPLE_ANTENNAE.clone()
            }
            .unique_antennae_in_bounds(0..)
            .len(),
            34
        );
    }
}
