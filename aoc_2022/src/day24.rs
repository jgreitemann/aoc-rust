use aoc_companion::prelude::*;
use aoc_utils::geometry::Point;
use aoc_utils::linalg::Vector;

use thiserror::Error;

use std::collections::{BTreeSet, BinaryHeap, HashSet};

pub(crate) struct Door {
    blizzards: Blizzards,
    shape: (usize, usize),
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Self {
        let (blizzards, shape) = parse_input(input);
        Self { blizzards, shape }
    }

    fn part1(&self) -> Result<u32, RuntimeError> {
        shortest_time_to_exit(&self.blizzards, self.shape, 500)
    }

    fn part2(&self) -> Result<u32, RuntimeError> {
        shortest_time_for_snack_recovery(&self.blizzards, self.shape, 500)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub(crate) enum RuntimeError {
    #[error("Could not find a path to the exit within the specified time box")]
    NoPathFoundInTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Blizzard {
    pos: Vector<i32, 2>,
    facing: Vector<i32, 2>,
}

impl Blizzard {
    fn pos_at_time(&self, time: u32, shape: (usize, usize)) -> Vector<i32, 2> {
        let mut p = self.pos + self.facing * time as i32;
        p[0] += shape.0 as i32 * (time as i32 / shape.0 as i32 + 1);
        p[1] += shape.1 as i32 * (time as i32 / shape.1 as i32 + 1);
        p[0] %= shape.0 as i32;
        p[1] %= shape.1 as i32;
        p
    }
}

type Blizzards = HashSet<Blizzard>;

fn parse_input(input: &str) -> (Blizzards, (usize, usize)) {
    let blizzard_bytes: Vec<&[u8]> = input
        .lines()
        .skip(1)
        .filter_map(|line| line.strip_prefix("#").and_then(|l| l.strip_suffix("#")))
        .map(str::as_bytes)
        .take_while(|line| line[0] != b'#')
        .collect();

    let shape = (
        blizzard_bytes.len(),
        blizzard_bytes.first().map(|l| l.len()).unwrap_or(0),
    );
    let blizzard_data: Vec<_> = blizzard_bytes.into_iter().flatten().collect();
    let map = ndarray::ArrayView2::from_shape(shape, &blizzard_data).unwrap();
    (
        map.indexed_iter()
            .filter_map(|((y, x), b)| {
                let facing = match b {
                    b'>' => Some(Vector([0, 1])),
                    b'<' => Some(Vector([0, -1])),
                    b'^' => Some(Vector([-1, 0])),
                    b'v' => Some(Vector([1, 0])),
                    _ => None,
                };
                facing.map(|facing| Blizzard {
                    pos: Vector([y, x]).try_cast_as::<i32>().unwrap(),
                    facing,
                })
            })
            .collect(),
        shape,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpaceTime {
    time: u32,
    point: Vector<i32, 2>,
}

impl Ord for SpaceTime {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Ord::cmp(&other.time, &self.time).then_with(|| Ord::cmp(&self.point, &other.point))
    }
}

impl PartialOrd for SpaceTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn shortest_time_for_path(
    start: SpaceTime,
    end: Vector<i32, 2>,
    blizzards: &Blizzards,
    shape: (usize, usize),
    time_box: u32,
) -> Result<u32, RuntimeError> {
    let final_time = start.time + time_box;
    let mut flows = ndarray::Array2::from_elem(shape, BTreeSet::new());
    let mut queue = BinaryHeap::from([start.clone()]);

    while let Some(SpaceTime {
        time,
        point: current,
    }) = queue.pop()
    {
        let mut neighbors: HashSet<_> = current
            .nearest_neighbors()
            .filter_map(|n| n.try_cast_as::<usize>().ok())
            .filter(|&Vector([y, x])| y < shape.0 && x < shape.1)
            .collect();

        for time in (time + 1..=final_time).take_while(|&t| {
            !blizzards
                .iter()
                .any(|b| b.pos_at_time(t - 1, shape) == current)
        }) {
            for n in neighbors.clone().iter() {
                let n_int = n.try_cast_as::<i32>().unwrap();
                if !blizzards
                    .iter()
                    .any(|b| b.pos_at_time(time, shape) == n_int)
                {
                    if current != start.point {
                        neighbors.remove(n);
                    }
                    if flows[*n].insert(time) {
                        queue.push(SpaceTime { time, point: n_int });
                    }
                }
            }
        }
    }

    flows[end.try_cast_as::<usize>().unwrap()]
        .first()
        .copied()
        .ok_or(RuntimeError::NoPathFoundInTime)
        .map(|t| t + 1)
}

const START_POINT: Vector<i32, 2> = Vector([-1, 0]);
const NEXT_TO_START_POINT: Vector<i32, 2> = Vector([0, 0]);

fn shortest_time_to_exit(
    blizzards: &Blizzards,
    shape: (usize, usize),
    time_box: u32,
) -> Result<u32, RuntimeError> {
    let next_to_exit = Vector([shape.0 - 1, shape.1 - 1]);
    shortest_time_for_path(
        SpaceTime {
            time: 0,
            point: START_POINT,
        },
        next_to_exit.try_cast_as().unwrap(),
        blizzards,
        shape,
        time_box,
    )
}

fn shortest_time_for_snack_recovery(
    blizzards: &Blizzards,
    shape: (usize, usize),
    time_box_per_leg: u32,
) -> Result<u32, RuntimeError> {
    let next_to_exit = Vector([shape.0 - 1, shape.1 - 1]).try_cast_as().unwrap();
    let exit = Vector([shape.0, shape.1 - 1]).try_cast_as().unwrap();
    shortest_time_for_path(
        SpaceTime {
            time: 0,
            point: START_POINT,
        },
        next_to_exit,
        blizzards,
        shape,
        time_box_per_leg,
    )
    .and_then(|time| {
        shortest_time_for_path(
            SpaceTime { time, point: exit },
            NEXT_TO_START_POINT,
            blizzards,
            shape,
            time_box_per_leg,
        )
    })
    .and_then(|time| {
        shortest_time_for_path(
            SpaceTime {
                time,
                point: START_POINT,
            },
            next_to_exit,
            blizzards,
            shape,
            time_box_per_leg,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_is_parsed() {
        assert_eq!(
            parse_input(EXAMPLE_INPUT),
            (HashSet::from(EXAMPLE_BLIZZARDS), EXAMPLE_SHAPE)
        );
    }

    #[test]
    fn blizzard_position_at_time() {
        assert_eq!(
            Blizzard {
                pos: Vector([0, 0]),
                facing: Vector([0, 1])
            }
            .pos_at_time(0, EXAMPLE_SHAPE),
            Vector([0, 0])
        );
        assert_eq!(
            Blizzard {
                pos: Vector([0, 0]),
                facing: Vector([0, 1])
            }
            .pos_at_time(1, EXAMPLE_SHAPE),
            Vector([0, 1])
        );
        assert_eq!(
            Blizzard {
                pos: Vector([0, 0]),
                facing: Vector([0, 1])
            }
            .pos_at_time(5, EXAMPLE_SHAPE),
            Vector([0, 5])
        );
        assert_eq!(
            Blizzard {
                pos: Vector([0, 0]),
                facing: Vector([0, 1])
            }
            .pos_at_time(6, EXAMPLE_SHAPE),
            Vector([0, 0])
        );

        assert_eq!(
            Blizzard {
                pos: Vector([0, 5]),
                facing: Vector([-1, 0])
            }
            .pos_at_time(1, EXAMPLE_SHAPE),
            Vector([3, 5])
        );
        assert_eq!(
            Blizzard {
                pos: Vector([0, 5]),
                facing: Vector([-1, 0])
            }
            .pos_at_time(9, EXAMPLE_SHAPE),
            Vector([3, 5])
        );
    }

    #[test]
    fn shortest_time_to_exit_is_found() {
        assert_eq!(
            shortest_time_to_exit(&HashSet::from(EXAMPLE_BLIZZARDS), EXAMPLE_SHAPE, 25),
            Ok(18)
        );
    }

    #[test]
    fn shortest_time_for_snack_recovery_is_found() {
        assert_eq!(
            shortest_time_for_snack_recovery(&HashSet::from(EXAMPLE_BLIZZARDS), EXAMPLE_SHAPE, 50),
            Ok(54)
        );
    }

    const EXAMPLE_INPUT: &str = "\
#.######
#>>.<^<#
#.<..<<#
#>v.><>#
#<^v^^>#
######.#";

    const EXAMPLE_SHAPE: (usize, usize) = (4, 6);

    const EXAMPLE_BLIZZARDS: [Blizzard; 19] = [
        Blizzard {
            pos: Vector([0, 0]),
            facing: Vector([0, 1]),
        },
        Blizzard {
            pos: Vector([0, 1]),
            facing: Vector([0, 1]),
        },
        Blizzard {
            pos: Vector([0, 3]),
            facing: Vector([0, -1]),
        },
        Blizzard {
            pos: Vector([0, 4]),
            facing: Vector([-1, 0]),
        },
        Blizzard {
            pos: Vector([0, 5]),
            facing: Vector([0, -1]),
        },
        Blizzard {
            pos: Vector([1, 1]),
            facing: Vector([0, -1]),
        },
        Blizzard {
            pos: Vector([1, 4]),
            facing: Vector([0, -1]),
        },
        Blizzard {
            pos: Vector([1, 5]),
            facing: Vector([0, -1]),
        },
        Blizzard {
            pos: Vector([2, 0]),
            facing: Vector([0, 1]),
        },
        Blizzard {
            pos: Vector([2, 1]),
            facing: Vector([1, 0]),
        },
        Blizzard {
            pos: Vector([2, 3]),
            facing: Vector([0, 1]),
        },
        Blizzard {
            pos: Vector([2, 4]),
            facing: Vector([0, -1]),
        },
        Blizzard {
            pos: Vector([2, 5]),
            facing: Vector([0, 1]),
        },
        Blizzard {
            pos: Vector([3, 0]),
            facing: Vector([0, -1]),
        },
        Blizzard {
            pos: Vector([3, 1]),
            facing: Vector([-1, 0]),
        },
        Blizzard {
            pos: Vector([3, 2]),
            facing: Vector([1, 0]),
        },
        Blizzard {
            pos: Vector([3, 3]),
            facing: Vector([-1, 0]),
        },
        Blizzard {
            pos: Vector([3, 4]),
            facing: Vector([-1, 0]),
        },
        Blizzard {
            pos: Vector([3, 5]),
            facing: Vector([0, 1]),
        },
    ];
}
