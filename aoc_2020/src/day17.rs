use std::collections::HashSet;

use aoc_companion::prelude::*;
use aoc_utils::{geometry::Point, linalg::Vector};
use itertools::Itertools as _;

pub(crate) struct Door {
    active_cubes: HashSet<Vector<i64, 3>>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        let plane = aoc_utils::geometry::parse_ascii_map(input)?;
        let offset = Vector([plane.dim().1 as i64 / 2, plane.dim().0 as i64 / 2, 0]);
        let active_cubes = plane
            .indexed_iter()
            .filter_map(|((y, x), &b)| (b == b'#').then_some(Vector([x, y]).try_cast_as()))
            .map_ok(|p| p.embed() - offset)
            .try_collect()?;
        Ok(Self { active_cubes })
    }

    fn part1(&self) -> usize {
        active_after_boot(self.active_cubes.clone())
    }

    fn part2(&self) -> usize {
        active_after_boot(self.active_cubes.iter().map(|v| v.embed::<4>()).collect())
    }
}

fn evolve<const N: usize>(active_cubes: &HashSet<Vector<i64, N>>) -> HashSet<Vector<i64, N>>
where
    Vector<i64, N>: Point,
{
    let mut new_cubes: HashSet<_> = active_cubes
        .iter()
        .copied()
        .filter(|p| (2..=3).contains(&p.neighbors().filter(|n| active_cubes.contains(n)).count()))
        .collect();
    new_cubes.extend(
        active_cubes
            .iter()
            .flat_map(|p| p.neighbors())
            .filter(|p| !active_cubes.contains(p))
            .filter(|p| p.neighbors().filter(|n| active_cubes.contains(n)).count() == 3),
    );
    new_cubes
}

fn active_after_boot<const N: usize>(active_cubes: HashSet<Vector<i64, N>>) -> usize
where
    Vector<i64, N>: Point,
{
    let after_boot = std::iter::successors(Some(active_cubes), |cubes| Some(evolve(cubes)))
        .nth(6)
        .unwrap();
    after_boot.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "\
.#.
..#
###";

    const EXAMPLE_CUBES: [Vector<i64, 3>; 5] = [
        Vector([0, -1, 0]),
        Vector([1, 0, 0]),
        Vector([-1, 1, 0]),
        Vector([0, 1, 0]),
        Vector([1, 1, 0]),
    ];

    const EXAMPLE_AFTER_1: &[&str] = &[
        "\
...
...
#..
..#
.#.",
        "\
...
...
#.#
.##
.#.",
        "\
...
...
#..
..#
.#.",
    ];

    const EXAMPLE_AFTER_2: &[&str] = &[
        "\
.....
.....
.....
.....
..#..
.....
.....",
        "\
.....
.....
..#..
.#..#
....#
.#...
.....",
        "\
.....
.....
##...
##...
#....
....#
.###.",
        "\
.....
.....
..#..
.#..#
....#
.#...
.....",
        "\
.....
.....
.....
.....
..#..
.....
.....",
    ];

    const EXAMPLE_AFTER_3: &[&str] = &[
        "\
.......
.......
.......
.......
..##...
..###..
.......
.......
.......",
        "\
.......
.......
..#....
...#...
#......
.....##
.#...#.
..#.#..
...#...",
        "\
.......
.......
...#...
.......
#......
.......
.....##
.##.#..
...#...",
        "\
.......
.......
..#....
...#...
#......
.....##
.#...#.
..#.#..
...#...",
        "\
.......
.......
.......
.......
..##...
..###..
.......
.......
.......",
    ];

    #[test]
    fn parse_example_input() {
        let Door { active_cubes } = Door::parse(EXAMPLE_INPUT).unwrap();
        assert_eq!(active_cubes, HashSet::from(EXAMPLE_CUBES));
    }

    fn from_z_layers(layers: &[&str]) -> HashSet<Vector<i64, 3>> {
        layers
            .iter()
            .enumerate()
            .flat_map(|(z, layer)| {
                let Door { active_cubes } = Door::parse(layer).unwrap();
                active_cubes
                    .into_iter()
                    .map(move |p| p + Vector([0, 0, z as i64 - layers.len() as i64 / 2]))
            })
            .collect()
    }

    #[test]
    fn evolve_example() {
        itertools::assert_equal(
            std::iter::successors(Some(HashSet::from(EXAMPLE_CUBES)), |cubes| {
                Some(evolve(cubes))
            })
            .take(4),
            [
                &[EXAMPLE_INPUT],
                EXAMPLE_AFTER_1,
                EXAMPLE_AFTER_2,
                EXAMPLE_AFTER_3,
            ]
            .map(from_z_layers),
        );
    }

    #[test]
    fn active_after_boot_in_3d() {
        assert_eq!(active_after_boot(HashSet::from(EXAMPLE_CUBES)), 112);
    }

    #[test]
    fn active_after_boot_in_4d() {
        assert_eq!(
            active_after_boot(HashSet::from(EXAMPLE_CUBES.map(|v| v.embed::<4>()))),
            848
        );
    }
}
