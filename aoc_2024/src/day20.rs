use std::collections::HashMap;

use aoc_companion::prelude::*;
use aoc_utils::{
    geometry::{Point, map_bounds},
    linalg::Vector,
};
use ndarray::ShapeError;

pub(crate) struct Door {
    map: Map,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ShapeError> {
        Ok(Door {
            map: parse_map(input)?,
        })
    }

    fn part1(&self) -> usize {
        cheats(2, &self.map).filter(|g| *g >= 100).count()
    }

    fn part2(&self) -> usize {
        cheats(20, &self.map).filter(|g| *g >= 100).count()
    }
}

type Map = ndarray::Array2<u8>;

fn parse_map(input: &str) -> Result<Map, ShapeError> {
    let shape = map_bounds(input).map(|b| b.end);
    let data = input
        .lines()
        .flat_map(|line| line.as_bytes())
        .copied()
        .collect();
    Map::from_shape_vec(shape, data)
}

fn race_track(map: &Map) -> impl Iterator<Item = Vector<usize, 2>> + use<'_> {
    let start = map
        .indexed_iter()
        .find(|(_, elem)| **elem == b'S')
        .map(|((x, y), _)| Vector([x, y]));

    std::iter::successors(start.map(|s| (None, s)), |(prev, current)| {
        current
            .nearest_neighbors()
            .filter(|n| &Some(*n) != prev)
            .find(|n| map.get(*n).is_some_and(|q| *q != b'#'))
            .map(|next| (Some(*current), next))
    })
    .map(|(_, p)| p)
}

fn cheats(duration: usize, map: &Map) -> impl Iterator<Item = usize> + use<'_> {
    let race_track: HashMap<_, _> = race_track(map).enumerate().map(|(d, p)| (p, d)).collect();
    let rc_race_track = std::rc::Rc::new(race_track.clone());

    race_track.into_iter().flat_map(move |(start, start_dist)| {
        let rc_race_track = rc_race_track.clone();
        let start = start.try_cast_as::<isize>().unwrap();
        (1..=duration as isize)
            .flat_map(move |d| {
                (0..d)
                    .flat_map(move |dd| {
                        [
                            Vector([dd, d - dd]),
                            Vector([d - dd, -dd]),
                            Vector([-dd, dd - d]),
                            Vector([dd - d, dd]),
                        ]
                    })
                    .map(move |v| start + v)
            })
            .filter_map(move |end| {
                end.try_cast_as::<usize>()
                    .ok()
                    .and_then(|e| rc_race_track.get(&e).copied())
                    .map(|d| (d, end))
            })
            .map(move |(end_dist, end)| {
                end_dist
                    .saturating_sub(start_dist)
                    .saturating_sub((start - end).norm_l1() as usize)
            })
            .filter(|&gain| gain > 0)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "\
###############
#...#...#.....#
#.#.#.#.#.###.#
#S#...#.#.#...#
#######.#.#.###
#######.#.#...#
#######.#.###.#
###..E#...#...#
###.#######.###
#...###...#...#
#.#####.#.###.#
#.#...#.#.#...#
#.#.#.#.#.#.###
#...#...#...###
###############";

    #[test]
    fn number_of_short_cheats_with_given_effectiveness_in_example() {
        let map = parse_map(EXAMPLE_INPUT).unwrap();
        fn number_of_cheats_with_effectiveness(gain: usize, map: &Map) -> usize {
            cheats(2, map).filter(|g| *g == gain).count()
        }
        assert_eq!(number_of_cheats_with_effectiveness(2, &map), 14);
        assert_eq!(number_of_cheats_with_effectiveness(3, &map), 0);
        assert_eq!(number_of_cheats_with_effectiveness(4, &map), 14);
        assert_eq!(number_of_cheats_with_effectiveness(6, &map), 2);
        assert_eq!(number_of_cheats_with_effectiveness(8, &map), 4);
        assert_eq!(number_of_cheats_with_effectiveness(10, &map), 2);
        assert_eq!(number_of_cheats_with_effectiveness(12, &map), 3);
        assert_eq!(number_of_cheats_with_effectiveness(20, &map), 1);
        assert_eq!(number_of_cheats_with_effectiveness(36, &map), 1);
        assert_eq!(number_of_cheats_with_effectiveness(38, &map), 1);
        assert_eq!(number_of_cheats_with_effectiveness(40, &map), 1);
        assert_eq!(number_of_cheats_with_effectiveness(64, &map), 1);
    }

    #[test]
    fn number_of_long_cheats_with_given_effectiveness_in_example() {
        let map = parse_map(EXAMPLE_INPUT).unwrap();
        fn number_of_cheats_with_effectiveness(gain: usize, map: &Map) -> usize {
            cheats(20, map).filter(|g| *g == gain).count()
        }
        assert_eq!(number_of_cheats_with_effectiveness(50, &map), 32);
        assert_eq!(number_of_cheats_with_effectiveness(52, &map), 31);
        assert_eq!(number_of_cheats_with_effectiveness(54, &map), 29);
        assert_eq!(number_of_cheats_with_effectiveness(56, &map), 39);
        assert_eq!(number_of_cheats_with_effectiveness(58, &map), 25);
        assert_eq!(number_of_cheats_with_effectiveness(60, &map), 23);
        assert_eq!(number_of_cheats_with_effectiveness(62, &map), 20);
        assert_eq!(number_of_cheats_with_effectiveness(64, &map), 19);
        assert_eq!(number_of_cheats_with_effectiveness(66, &map), 12);
        assert_eq!(number_of_cheats_with_effectiveness(68, &map), 14);
        assert_eq!(number_of_cheats_with_effectiveness(70, &map), 12);
        assert_eq!(number_of_cheats_with_effectiveness(72, &map), 22);
        assert_eq!(number_of_cheats_with_effectiveness(74, &map), 4);
        assert_eq!(number_of_cheats_with_effectiveness(76, &map), 3);
    }
}
