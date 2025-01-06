use std::collections::HashSet;

use anyhow::{anyhow, bail};
use aoc_companion::prelude::*;
use aoc_utils::{
    geometry::{try_parse_map, ParseMapError},
    linalg::Vector,
};
use itertools::Itertools;

pub(crate) struct Door {
    map: Map,
    moves: Vec<Move>,
}

type Map = ndarray::Array2<Tile>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tile {
    Free,
    Wall,
    Crate,
    LCrate,
    RCrate,
    Robot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Move {
    Up,
    Left,
    Down,
    Right,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        let Some((map_str, moves_str)) = input.split_once("\n\n") else {
            bail!("Missing empty line separating maps from moves");
        };

        Ok(Door {
            map: parse_map(map_str)?,
            moves: parse_moves(moves_str)?,
        })
    }

    fn part1(&self) -> usize {
        let mut map = self.map.clone();
        for &mv in &self.moves {
            move_robot(&mut map, mv);
        }
        gps_sum(&map)
    }

    fn part2(&self) -> usize {
        let mut map = scale_up(&self.map);
        for &mv in &self.moves {
            move_robot(&mut map, mv);
        }
        gps_sum(&map)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Unrecognized tile: {0:#x}")]
struct UnrecognizedTile(u8);

fn parse_map(map_str: &str) -> Result<Map, ParseMapError<UnrecognizedTile>> {
    try_parse_map(map_str, |b| match b {
        b'.' => Ok(Tile::Free),
        b'#' => Ok(Tile::Wall),
        b'O' => Ok(Tile::Crate),
        b'[' => Ok(Tile::LCrate),
        b']' => Ok(Tile::RCrate),
        b'@' => Ok(Tile::Robot),
        b => Err(UnrecognizedTile(b)),
    })
}

fn parse_moves(moves_str: &str) -> Result<Vec<Move>> {
    moves_str
        .lines()
        .flat_map(|line| line.as_bytes())
        .map(|b| match b {
            b'^' => Ok(Move::Up),
            b'<' => Ok(Move::Left),
            b'v' => Ok(Move::Down),
            b'>' => Ok(Move::Right),
            b => Err(anyhow!("Unrecognized move: {b:?}")),
        })
        .try_collect()
}

fn find_robot(map: &Map) -> Vector<usize, 2> {
    map.indexed_iter()
        .find_map(|((x, y), &tile)| (tile == Tile::Robot).then_some(Vector([x, y])))
        .expect("robot must be somewhere on the map")
}

fn shift(p: Vector<usize, 2>, mv: Move) -> Option<Vector<usize, 2>> {
    (p.try_cast_as::<isize>().unwrap() + mv.unit_vec())
        .try_cast_as::<usize>()
        .ok()
}

fn move_robot(map: &mut Map, mv: Move) {
    let carousel_pos = std::iter::successors(Some(HashSet::from([find_robot(map)])), |current| {
        Some(
            current
                .iter()
                .filter_map(|c| shift(*c, mv))
                .filter(|p| map.get(*p).is_some_and(|&t| t != Tile::Free))
                .flat_map(|p| match (map[p], mv) {
                    (Tile::LCrate, Move::Up | Move::Down) => [Some(p), shift(p, Move::Right)],
                    (Tile::RCrate, Move::Up | Move::Down) => [shift(p, Move::Left), Some(p)],
                    _ => [Some(p), None],
                })
                .flatten()
                .collect(),
        )
    })
    .take_while(|front| front.iter().all(|p| map[*p] != Tile::Wall))
    .take_while_inclusive(|front| {
        front
            .iter()
            .any(|p| map[shift(*p, mv).unwrap()] != Tile::Free)
    })
    .collect_vec();

    if carousel_pos
        .last()
        .unwrap()
        .iter()
        .all(|p| map[shift(*p, mv).unwrap()] != Tile::Wall)
    {
        for front in carousel_pos.iter().rev() {
            for p in front {
                let t = std::mem::replace(map.get_mut(*p).unwrap(), Tile::Free);
                map[shift(*p, mv).unwrap()] = t;
            }
        }
    }
}

impl Move {
    fn unit_vec(&self) -> Vector<isize, 2> {
        match self {
            Move::Up => Vector([-1, 0]),
            Move::Left => Vector([0, -1]),
            Move::Down => Vector([1, 0]),
            Move::Right => Vector([0, 1]),
        }
    }
}

fn gps_sum(map: &Map) -> usize {
    map.indexed_iter()
        .filter(|(_, tile)| matches!(tile, Tile::Crate | Tile::LCrate))
        .map(|((x, y), _)| 100 * x + y)
        .sum()
}

fn scale_up(map: &Map) -> Map {
    let shape = [map.shape()[0], map.shape()[1] * 2];

    let data = map
        .iter()
        .flat_map(|tile| match tile {
            Tile::Free => [Tile::Free, Tile::Free],
            Tile::Wall => [Tile::Wall, Tile::Wall],
            Tile::Crate => [Tile::LCrate, Tile::RCrate],
            Tile::Robot => [Tile::Robot, Tile::Free],
            Tile::LCrate | Tile::RCrate => unreachable!(),
        })
        .collect();

    Map::from_shape_vec(shape, data).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SMALL_EXAMPLE_MAP_INPUT: &str = "\
########
#..O.O.#
##@.O..#
#...O..#
#.#.O..#
#...O..#
#......#
########";

    const SMALL_EXAMPLE_MOVES_INPUT: &str = "<^^>>>vv<v>>v<<";

    const LARGER_EXAMPLE_MAP_INPUT: &str = "\
##########
#..O..O.O#
#......O.#
#.OO..O.O#
#..O@..O.#
#O#..O...#
#O..O..O.#
#.OO.O.OO#
#....O...#
##########";

    const LARGER_EXAMPLE_MOVES_INPUT: &str = "\
<vv>^<v^>v>^vv^v>v<>v^v<v<^vv<<<^><<><>>v<vvv<>^v^>^<<<><<v<<<v^vv^v>^
vvv<<^>^v^^><<>>><>^<<><^vv^^<>vvv<>><^^v>^>vv<>v<<<<v<^v>^<^^>>>^<v<v
><>vv>v^v^<>><>>>><^^>vv>v<^^^>>v^v^<^^>v^^>v^<^v>v<>>v^v^<v>v^^<^^vv<
<<v<^>>^^^^>>>v^<>vvv^><v<<<>^^^vv^<vvv>^>v<^^^^v<>^>vvvv><>>v^<<^^^^^
^><^><>>><>^^<<^^v>>><^<v>^<vv>>v>>>^v><>^v><<<<v>>v<v<v>vvv>^<><<>^><
^>><>^v<><^vvv<^^<><v<<<<<><^v<<<><<<^^<v<^^^><^>>^<v^><<<^>>^v<v^v<v^
>^>>^v>vv>^<<^v<>><<><<v<<v><>v<^vv<<<>^^v^>^^>>><<^v>>v^v><^^>>^<>vv^
<><^^>^^^<><vvvvv^v<v<<>^v<v>v<<^><<><<><<<^^<<<^<<>><<><^^^>^^<>^>v<>
^^>vv<^v^v<vv>^<><v<^v>^^^>>>^^vvv^>vvv<>>>^<^>>>>>^<<^v>^vvv<>^<><<v>
v^^>>><<^^<>>^v^<v^vv<>v^<<>^<^v^v><^<<<><<^<v><v<>vv>>v><v^<vv<>v^<<^";

    #[test]
    fn map_after_moves_for_small_example() {
        let mut map = parse_map(SMALL_EXAMPLE_MAP_INPUT).unwrap();

        move_robot(&mut map, Move::Left);
        assert_eq!(
            map,
            parse_map(
                "\
########
#..O.O.#
##@.O..#
#...O..#
#.#.O..#
#...O..#
#......#
########"
            )
            .unwrap()
        );

        move_robot(&mut map, Move::Up);
        assert_eq!(
            map,
            parse_map(
                "\
########
#.@O.O.#
##..O..#
#...O..#
#.#.O..#
#...O..#
#......#
########"
            )
            .unwrap()
        );

        move_robot(&mut map, Move::Up);
        assert_eq!(
            map,
            parse_map(
                "\
########
#.@O.O.#
##..O..#
#...O..#
#.#.O..#
#...O..#
#......#
########"
            )
            .unwrap()
        );

        move_robot(&mut map, Move::Right);
        assert_eq!(
            map,
            parse_map(
                "\
########
#..@OO.#
##..O..#
#...O..#
#.#.O..#
#...O..#
#......#
########"
            )
            .unwrap()
        );
    }

    #[test]
    fn gps_sum_for_small_example() {
        let mut map = parse_map(SMALL_EXAMPLE_MAP_INPUT).unwrap();
        let moves = parse_moves(SMALL_EXAMPLE_MOVES_INPUT).unwrap();
        for mv in moves {
            move_robot(&mut map, mv);
        }
        assert_eq!(gps_sum(&map), 2028);
    }

    #[test]
    fn gps_sum_for_larger_example() {
        let mut map = parse_map(LARGER_EXAMPLE_MAP_INPUT).unwrap();
        let moves = parse_moves(LARGER_EXAMPLE_MOVES_INPUT).unwrap();
        for mv in moves {
            move_robot(&mut map, mv);
        }
        assert_eq!(gps_sum(&map), 10092);
    }

    const SCALED_UP_EXAMPLE_INPUT: &str = "\
####################
##....[]....[]..[]##
##............[]..##
##..[][]....[]..[]##
##....[]@.....[]..##
##[]##....[]......##
##[]....[]....[]..##
##..[][]..[]..[][]##
##........[]......##
####################";

    const SCALED_UP_EXAMPLE_FINAL: &str = "\
####################
##[].......[].[][]##
##[]...........[].##
##[]........[][][]##
##[]......[]....[]##
##..##......[]....##
##..[]............##
##..@......[].[][]##
##......[][]..[]..##
####################";

    #[test]
    fn large_example_map_is_scaled_up() {
        assert_eq!(
            scale_up(&parse_map(LARGER_EXAMPLE_MAP_INPUT).unwrap()),
            parse_map(SCALED_UP_EXAMPLE_INPUT).unwrap(),
        );
    }

    #[test]
    fn final_state_of_scaled_up_larger_example() {
        let mut map = scale_up(&parse_map(LARGER_EXAMPLE_MAP_INPUT).unwrap());
        let moves = parse_moves(LARGER_EXAMPLE_MOVES_INPUT).unwrap();
        for mv in moves {
            move_robot(&mut map, mv);
        }
        assert_eq!(map, parse_map(SCALED_UP_EXAMPLE_FINAL).unwrap(),);
    }

    #[test]
    fn gps_sum_for_scaled_up_larger_example() {
        assert_eq!(gps_sum(&parse_map(SCALED_UP_EXAMPLE_FINAL).unwrap()), 9021);
    }
}
