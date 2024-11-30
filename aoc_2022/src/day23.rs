use aoc_companion::prelude::*;
use aoc_utils::geometry::Point;
use aoc_utils::linalg::Vector;

use itertools::Itertools;

use std::collections::{HashMap, HashSet};
use std::ops::Range;

pub(crate) struct Door {
    elves: Coords,
}

impl<'input> ParseInput<'input> for Door {
    fn parse(input: &'input str) -> Self {
        Self {
            elves: parse_input(input),
        }
    }
}

impl Part1 for Door {
    fn part1(&self) -> usize {
        open_spaces_in_bounding_rect(&execute_many_rounds(self.elves.clone(), 10).0)
    }
}

impl Part2 for Door {
    fn part2(&self) -> usize {
        execute_many_rounds(self.elves.clone(), usize::MAX).1
    }
}

type Coord = Vector<i32, 2>;
type Coords = HashSet<Coord>;

fn parse_input(input: &str) -> Coords {
    let v: Vec<_> = input
        .lines()
        .flat_map(|line| line.as_bytes().iter())
        .collect();
    let shape = (
        input.lines().count(),
        input.lines().next().map(str::len).unwrap_or(0),
    );
    let map = ndarray::ArrayView2::from_shape(shape, v.as_slice()).unwrap();

    map.indexed_iter()
        .filter_map(|((y, x), &&c)| (c == b'#').then_some(Vector([x as i32, y as i32])))
        .collect()
}

fn open_spaces_in_bounding_rect(coords: &Coords) -> usize {
    let bounding_width = coords
        .iter()
        .map(|c| c[0])
        .minmax()
        .into_option()
        .map(|(min, max)| max - min + 1)
        .unwrap_or(0) as usize;
    let bounding_height = coords
        .iter()
        .map(|c| c[1])
        .minmax()
        .into_option()
        .map(|(min, max)| max - min + 1)
        .unwrap_or(0) as usize;
    bounding_width * bounding_height - coords.len()
}

fn execute_round(elves: &Coords, priorities: &[Range<usize>]) -> Coords {
    let propositions: HashMap<Coord, Coord> = elves
        .iter()
        .map(|elf| {
            let mut neighbors = elf.neighbors().collect_vec();
            neighbors.rotate_left(1);
            neighbors.push(*neighbors.first().unwrap());

            if neighbors.iter().all(|n| !elves.contains(n)) {
                (*elf, *elf)
            } else {
                let direction = priorities
                    .iter()
                    .find(|&prio| neighbors[prio.clone()].iter().all(|n| !elves.contains(n)));

                (
                    *elf,
                    direction.map(|p| neighbors[p.clone()][1]).unwrap_or(*elf),
                )
            }
        })
        .collect();

    let proposition_counts = propositions.values().copied().counts();

    propositions
        .into_iter()
        .map(|(from, to)| {
            if proposition_counts[&to] == 1 {
                to
            } else {
                from
            }
        })
        .collect()
}

fn execute_many_rounds(mut elves: Coords, n: usize) -> (Coords, usize) {
    let mut priorities = vec![4..7, 0..3, 2..5, 6..9];
    let mut round = 0;
    loop {
        let new_elves = execute_round(&elves, &priorities);
        if new_elves == elves || round == n {
            break (elves, round + 1);
        }
        elves = new_elves;
        priorities.rotate_left(1);
        round += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_is_parsed() {
        assert_eq!(
            parse_input(SMALL_EXAMPLE_INPUT),
            HashSet::from(SMALL_EXAMPLE_COORDS)
        );
    }

    #[test]
    fn number_of_open_spaces_in_bounding_rect_is_determined() {
        assert_eq!(
            open_spaces_in_bounding_rect(&HashSet::from(SMALL_EXAMPLE_COORDS)),
            3
        );
        assert_eq!(
            open_spaces_in_bounding_rect(&HashSet::from(SMALL_EXAMPLE_COORDS_ROUNDS[2])),
            25
        );
        assert_eq!(
            open_spaces_in_bounding_rect(&parse_input(MEDIUM_EXAMPLE_AFTER_TEN_ROUNDS)),
            110
        );
    }

    #[test]
    fn small_example_rounds() {
        assert_eq!(
            execute_round(
                &HashSet::from(SMALL_EXAMPLE_COORDS),
                &[4..7, 0..3, 2..5, 6..9]
            ),
            HashSet::from(SMALL_EXAMPLE_COORDS_ROUNDS[0])
        );
        assert_eq!(
            execute_round(
                &HashSet::from(SMALL_EXAMPLE_COORDS_ROUNDS[0]),
                &[0..3, 2..5, 6..9, 4..7]
            ),
            HashSet::from(SMALL_EXAMPLE_COORDS_ROUNDS[1])
        );
        assert_eq!(
            execute_round(
                &HashSet::from(SMALL_EXAMPLE_COORDS_ROUNDS[1]),
                &[2..5, 6..9, 4..7, 0..3]
            ),
            HashSet::from(SMALL_EXAMPLE_COORDS_ROUNDS[2])
        );
    }

    #[test]
    fn medium_example_rounds() {
        assert_eq!(
            execute_round(
                &parse_input(MEDIUM_EXAMPLE_INPUT),
                &[4..7, 0..3, 2..5, 6..9]
            ),
            parse_input(MEDIUM_EXAMPLE_ROUNDS[0])
        );
        assert_eq!(
            execute_round(
                &parse_input(MEDIUM_EXAMPLE_ROUNDS[0]),
                &[0..3, 2..5, 6..9, 4..7]
            ),
            parse_input(MEDIUM_EXAMPLE_ROUNDS[1])
        );
        assert_eq!(
            execute_round(
                &parse_input(MEDIUM_EXAMPLE_ROUNDS[1]),
                &[2..5, 6..9, 4..7, 0..3]
            ),
            parse_input(MEDIUM_EXAMPLE_ROUNDS[2])
        );
        assert_eq!(
            execute_round(
                &parse_input(MEDIUM_EXAMPLE_ROUNDS[2]),
                &[6..9, 4..7, 0..3, 2..5]
            ),
            parse_input(MEDIUM_EXAMPLE_ROUNDS[3])
        );
    }

    #[test]
    fn final_elf_configuration_is_reached() {
        assert_eq!(
            execute_many_rounds(HashSet::from(SMALL_EXAMPLE_COORDS), 10).0,
            HashSet::from(SMALL_EXAMPLE_COORDS_ROUNDS[2])
        );
        assert_eq!(
            execute_many_rounds(parse_input(MEDIUM_EXAMPLE_INPUT), 10).0,
            parse_input(MEDIUM_EXAMPLE_AFTER_TEN_ROUNDS)
        );
    }

    #[test]
    fn number_of_final_round_is_found() {
        assert_eq!(
            execute_many_rounds(HashSet::from(SMALL_EXAMPLE_COORDS), usize::MAX).1,
            4
        );
        assert_eq!(
            execute_many_rounds(parse_input(MEDIUM_EXAMPLE_INPUT), usize::MAX).1,
            20
        );
    }

    const SMALL_EXAMPLE_INPUT: &str = "\
##
#.
..
##";

    const SMALL_EXAMPLE_COORDS: [Coord; 5] = [
        Vector([0, 0]),
        Vector([1, 0]),
        Vector([0, 1]),
        Vector([0, 3]),
        Vector([1, 3]),
    ];

    const SMALL_EXAMPLE_COORDS_ROUNDS: [[Coord; 5]; 3] = [
        //..##.
        //.....
        //..#..
        //...#.
        //..#..
        //.....
        [
            Vector([0, -1]),
            Vector([1, -1]),
            Vector([0, 1]),
            Vector([0, 3]),
            Vector([1, 2]),
        ],
        //.....
        //..##.
        //.#...
        //....#
        //.....
        //..#..
        [
            Vector([0, 0]),
            Vector([1, 0]),
            Vector([-1, 1]),
            Vector([0, 4]),
            Vector([2, 2]),
        ],
        //..#..
        //....#
        //#....
        //....#
        //.....
        //..#..
        [
            Vector([0, -1]),
            Vector([2, 0]),
            Vector([-2, 1]),
            Vector([0, 4]),
            Vector([2, 2]),
        ],
    ];

    const MEDIUM_EXAMPLE_INPUT: &str = "\
..............
..............
.......#......
.....###.#....
...#...#.#....
....#...##....
...#.###......
...##.#.##....
....#..#......
..............
..............
..............";

    const MEDIUM_EXAMPLE_ROUNDS: [&str; 5] = [
        "\
..............
.......#......
.....#...#....
...#..#.#.....
.......#..#...
....#.#.##....
..#..#.#......
..#.#.#.##....
..............
....#..#......
..............
..............",
        "\
..............
.......#......
....#.....#...
...#..#.#.....
.......#...#..
...#..#.#.....
.#...#.#.#....
..............
..#.#.#.##....
....#..#......
..............
..............",
        "\
..............
.......#......
.....#....#...
..#..#...#....
.......#...#..
...#..#.#.....
.#..#.....#...
.......##.....
..##.#....#...
...#..........
.......#......
..............",
        "\
..............
.......#......
......#....#..
..#...##......
...#.....#.#..
.........#....
.#...###..#...
..#......#....
....##....#...
....#.........
.......#......
..............",
        "\
.......#......
..............
..#..#.....#..
.........#....
......##...#..
.#.#.####.....
...........#..
....##..#.....
..#...........
..........#...
....#..#......
..............",
    ];

    const MEDIUM_EXAMPLE_AFTER_TEN_ROUNDS: &str = "\
.......#......
...........#..
..#.#..#......
......#.......
...#.....#..#.
.#......##....
.....##.......
..#........#..
....#.#..#....
..............
....#..#..#...
..............";
}
