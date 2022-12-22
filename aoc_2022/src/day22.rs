use aoc_companion::prelude::*;
use aoc_utils::linalg::Vector;

use itertools::Itertools;
use thiserror::Error;

use std::num::ParseIntError;
use std::str::FromStr;

pub struct Door {
    map: Map,
    instructions: Vec<Instruction>,
}

impl ParseInput<'_> for Door {
    type Error = ParseError;

    fn parse(input: &str) -> Result<Self, Self::Error> {
        let (map_str, instr_str) = input
            .split_once("\n\n")
            .ok_or(ParseError::EmptyLineNotFound)?;
        Ok(Door {
            map: map_str.parse()?,
            instructions: parse_instructions(instr_str)?,
        })
    }
}

impl Part1 for Door {
    type Output = usize;
    type Error = std::convert::Infallible;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        Ok(self
            .map
            .player_start()
            .end::<PlainWrapping>(&self.instructions, &self.map)
            .password())
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Input does not contain an empty line")]
    EmptyLineNotFound,
    #[error(transparent)]
    ParseIntError(ParseIntError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Limit {
    min: usize,
    max: usize,
}

impl Limit {
    fn from_lane<'a, T>(iter: T) -> Self
    where
        T: IntoIterator<Item = &'a Tile> + Clone + 'a,
        <T as IntoIterator>::IntoIter: ExactSizeIterator + DoubleEndedIterator,
    {
        let min = iter
            .clone()
            .into_iter()
            .position(|&t| t != Tile::Nothing)
            .unwrap();
        let max = iter.into_iter().rposition(|&t| t != Tile::Nothing).unwrap();
        Limit { min, max }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    Nothing,
    Open,
    Wall,
}

impl From<u8> for Tile {
    fn from(c: u8) -> Self {
        match c {
            b'.' => Tile::Open,
            b'#' => Tile::Wall,
            _ => Tile::Nothing,
        }
    }
}

struct Map {
    data: ndarray::Array2<Tile>,
}

impl Map {
    fn player_start(&self) -> Player {
        Player {
            pos: self
                .data
                .indexed_iter()
                .find_map(|((x, y), &d)| (d == Tile::Open).then(|| Vector([x, y])))
                .unwrap(),
            facing: Direction::Right,
        }
    }
}

impl FromStr for Map {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let shape = (
            s.lines().count(),
            s.lines().map(str::len).max().unwrap_or(0),
        );

        let data = s
            .lines()
            .flat_map(|line| {
                line.as_bytes()
                    .iter()
                    .map(|&b| b.into())
                    .chain(std::iter::repeat(Tile::Nothing))
                    .take(shape.1)
            })
            .collect();

        let data = ndarray::Array2::from_shape_vec(shape, data).unwrap();

        Ok(Map { data })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Right,
    Down,
    Left,
    Up,
}

impl Direction {
    fn turn(&mut self, dir: Direction) {
        use Direction::*;
        match dir {
            Right => {
                *self = match self {
                    Right => Down,
                    Down => Left,
                    Left => Up,
                    Up => Right,
                }
            }
            Left => {
                *self = match self {
                    Right => Up,
                    Down => Right,
                    Left => Down,
                    Up => Left,
                }
            }
            _ => {}
        }
    }

    fn unit_vector(&self) -> Vector<isize, 2> {
        match self {
            Direction::Right => Vector([0, 1]),
            Direction::Down => Vector([1, 0]),
            Direction::Left => Vector([0, -1]),
            Direction::Up => Vector([-1, 0]),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Player {
    pos: Vector<usize, 2>,
    facing: Direction,
}

trait Wrapping {
    fn from_map(map: &Map) -> Self;
    fn advance(&self, player: Player) -> Player;
}

struct PlainWrapping {
    row_limits: Vec<Limit>,
    col_limits: Vec<Limit>,
}

impl PlainWrapping {
    fn wrap_h(&self, mut p: Vector<isize, 2>) -> Vector<usize, 2> {
        let range = self.row_limits[p[0] as usize].max - self.row_limits[p[0] as usize].min + 1;
        p[1] = p[1] - self.row_limits[p[0] as usize].min as isize + range as isize;
        p[1] %= range as isize;
        p[1] += self.row_limits[p[0] as usize].min as isize;

        p.try_cast_as().unwrap()
    }

    fn wrap_v(&self, mut p: Vector<isize, 2>) -> Vector<usize, 2> {
        let range = self.col_limits[p[1] as usize].max - self.col_limits[p[1] as usize].min + 1;
        p[0] = p[0] - self.col_limits[p[1] as usize].min as isize + range as isize;
        p[0] %= range as isize;
        p[0] += self.col_limits[p[1] as usize].min as isize;

        p.try_cast_as().unwrap()
    }
}

impl Wrapping for PlainWrapping {
    fn from_map(map: &Map) -> Self {
        PlainWrapping {
            row_limits: map.data.rows().into_iter().map(Limit::from_lane).collect(),
            col_limits: map
                .data
                .columns()
                .into_iter()
                .map(Limit::from_lane)
                .collect(),
        }
    }

    fn advance(&self, mut player: Player) -> Player {
        let new_pos = player.pos.try_cast_as::<isize>().unwrap() + player.facing.unit_vector();
        player.pos = match player.facing {
            Direction::Right | Direction::Left => self.wrap_h(new_pos),
            Direction::Down | Direction::Up => self.wrap_v(new_pos),
        };
        player
    }
}

impl Player {
    fn execute<W>(&mut self, instruction: Instruction, map: &Map, wrapping: &W)
    where
        W: Wrapping,
    {
        match instruction {
            Instruction::Move(by) => {
                for _ in 0..by {
                    let wrapped = wrapping.advance(self.clone());
                    if map.data[wrapped.pos] == Tile::Wall {
                        break;
                    } else {
                        *self = wrapped;
                    }
                }
            }
            Instruction::Turn(dir) => self.facing.turn(dir),
        }
    }

    fn end<W: Wrapping>(mut self, instructions: &[Instruction], map: &Map) -> Self {
        let wrapping = W::from_map(map);
        for instruction in instructions {
            self.execute(*instruction, map, &wrapping);
        }
        self
    }

    fn password(&self) -> usize {
        1000 * (self.pos[0] + 1)
            + 4 * (self.pos[1] + 1)
            + match self.facing {
                Direction::Right => 0,
                Direction::Down => 1,
                Direction::Left => 2,
                Direction::Up => 3,
            }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Instruction {
    Move(usize),
    Turn(Direction),
}

fn parse_instructions(s: &str) -> Result<Vec<Instruction>, ParseError> {
    s.chars()
        .into_iter()
        .group_by(|&b| b == 'L' || b == 'R')
        .into_iter()
        .flat_map(|(is_turn, group)| {
            if is_turn {
                group
                    .map(|b| match b {
                        'L' => Ok(Instruction::Turn(Direction::Left)),
                        'R' => Ok(Instruction::Turn(Direction::Right)),
                        _ => unreachable!(),
                    })
                    .collect()
            } else {
                let str = String::from_iter(group);
                vec![str
                    .parse()
                    .map(|by| Instruction::Move(by))
                    .map_err(|e| ParseError::ParseIntError(e))]
            }
        })
        .try_collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_is_parsed() {
        let Door { map, .. } = Door::parse(EXAMPLE_INPUT).unwrap();
        assert_eq!(map.data.shape(), [12, 16]);

        assert_eq!(map.data[Vector([0, 0])], Tile::Nothing);
        assert_eq!(map.data[Vector([0, 7])], Tile::Nothing);
        assert_eq!(map.data[Vector([0, 8])], Tile::Open);
        assert_eq!(map.data[Vector([0, 11])], Tile::Wall);
        assert_eq!(map.data[Vector([0, 12])], Tile::Nothing);

        assert_eq!(map.data[Vector([0, 3])], Tile::Nothing);
        assert_eq!(map.data[Vector([3, 3])], Tile::Nothing);
        assert_eq!(map.data[Vector([4, 3])], Tile::Wall);
        assert_eq!(map.data[Vector([7, 3])], Tile::Open);
        assert_eq!(map.data[Vector([8, 3])], Tile::Nothing);
    }

    #[test]
    fn instructions_are_parsed() {
        let Door { instructions, .. } = Door::parse(EXAMPLE_INPUT).unwrap();
        assert_eq!(instructions, EXAMPLE_INSTRUCTIONS);
    }

    #[test]
    fn starting_position_of_player_is_determined() {
        let Door { map, .. } = Door::parse(EXAMPLE_INPUT).unwrap();
        assert_eq!(
            map.player_start(),
            Player {
                pos: Vector([0, 8]),
                facing: Direction::Right
            }
        );
    }

    #[test]
    fn plain_wrapping_limits_are_found() {
        let Door { map, .. } = Door::parse(EXAMPLE_INPUT).unwrap();
        let wrapping = PlainWrapping::from_map(&map);

        assert_eq!(wrapping.row_limits[0], Limit { min: 8, max: 11 });
        assert_eq!(wrapping.col_limits[3], Limit { min: 4, max: 7 });
    }

    #[test]
    fn plain_wrap_points() {
        let Door { map, .. } = Door::parse(EXAMPLE_INPUT).unwrap();
        let wrapping = PlainWrapping::from_map(&map);
        assert_eq!(wrapping.wrap_h(Vector([0, 7])), Vector([0, 11]));
        assert_eq!(wrapping.wrap_h(Vector([0, 8])), Vector([0, 8]));
        assert_eq!(wrapping.wrap_h(Vector([0, 11])), Vector([0, 11]));
        assert_eq!(wrapping.wrap_h(Vector([0, 12])), Vector([0, 8]));

        assert_eq!(wrapping.wrap_v(Vector([3, 3])), Vector([7, 3]));
        assert_eq!(wrapping.wrap_v(Vector([4, 3])), Vector([4, 3]));
        assert_eq!(wrapping.wrap_v(Vector([7, 3])), Vector([7, 3]));
        assert_eq!(wrapping.wrap_v(Vector([8, 3])), Vector([4, 3]));

        assert_eq!(wrapping.wrap_h(Vector([3, 7])), Vector([3, 11]));
        assert_eq!(wrapping.wrap_v(Vector([3, 7])), Vector([7, 7]));

        assert_eq!(wrapping.wrap_h(Vector([5, 0])), Vector([5, 0]));
        assert_eq!(wrapping.wrap_h(Vector([5, -1])), Vector([5, 11]));
        assert_eq!(wrapping.wrap_h(Vector([5, 12])), Vector([5, 0]));
    }

    #[test]
    fn final_player_position_for_plain_wrapping() {
        let Door { map, .. } = Door::parse(EXAMPLE_INPUT).unwrap();
        let player = map
            .player_start()
            .end::<PlainWrapping>(EXAMPLE_INSTRUCTIONS, &map);
        assert_eq!(player, EXAMPLE_FINAL_PLAYER);
    }

    #[test]
    fn final_password_is_found_for_plain_wrapping() {
        assert_eq!(EXAMPLE_FINAL_PLAYER.password(), 6032);
    }

    const EXAMPLE_INPUT: &str = r"        ...#
        .#..
        #...
        ....
...#.......#
........#...
..#....#....
..........#.
        ...#....
        .....#..
        .#......
        ......#.

10R5L5R10L4R5L5";

    const EXAMPLE_INSTRUCTIONS: &[Instruction] = &[
        Instruction::Move(10),
        Instruction::Turn(Direction::Right),
        Instruction::Move(5),
        Instruction::Turn(Direction::Left),
        Instruction::Move(5),
        Instruction::Turn(Direction::Right),
        Instruction::Move(10),
        Instruction::Turn(Direction::Left),
        Instruction::Move(4),
        Instruction::Turn(Direction::Right),
        Instruction::Move(5),
        Instruction::Turn(Direction::Left),
        Instruction::Move(5),
    ];

    const EXAMPLE_FINAL_PLAYER: Player = Player {
        pos: Vector([5, 7]),
        facing: Direction::Right,
    };
}
