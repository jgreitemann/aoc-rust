use std::collections::{HashMap, HashSet};

use aoc_companion::prelude::*;
use aoc_utils::{
    array,
    geometry::{map_bounds, Point},
    linalg::Vector,
};

type Map = ndarray::Array2<u8>;

pub(crate) struct Door {
    graph: Graph,
    start: Vector<usize, 2>,
    end: Vector<usize, 2>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        let map = parse_map(input)?;
        let start = find_in_map(&map, b'S').unwrap();
        let end = find_in_map(&map, b'E').unwrap();
        Ok(Self {
            graph: graph(&map, start, end),
            start,
            end,
        })
    }

    fn part1(&self) -> usize {
        find_shortest_path(&self.graph, self.start, self.end)
    }

    fn part2(&self) -> usize {
        find_seats(&self.graph, self.start, self.end).seats.len() + 1
    }
}

fn parse_map(input: &str) -> Result<Map> {
    let bounds = map_bounds(input).map(|r| r.end);
    let data = input.lines().flat_map(str::as_bytes).copied().collect();
    Ok(Map::from_shape_vec(bounds, data)?)
}

fn find_in_map(map: &Map, target: u8) -> Option<Vector<usize, 2>> {
    map.indexed_iter()
        .find_map(|((row, col), b)| (*b == target).then_some(Vector([row, col])))
}

type Graph = HashMap<Vertex, Node>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Vertex {
    position: Vector<usize, 2>,
    facing: usize,
}

type Node = [Option<Edge>; 4];

#[derive(Debug, Clone, PartialEq, Eq)]
struct Edge {
    target: Vertex,
    distance: usize,
    seats: HashSet<Vector<usize, 2>>,
}

fn graph(map: &Map, start: Vector<usize, 2>, end: Vector<usize, 2>) -> Graph {
    let mut junctions = HashSet::from([start, end]);
    junctions.extend(map.indexed_iter().filter_map(|((row, col), b)| {
        (*b == b'.')
            .then_some(Vector([row, col]))
            .filter(|p| p.nearest_neighbors().filter(|n| map[*n] != b'#').count() > 2)
    }));

    junctions
        .iter()
        .flat_map(|p| {
            (0..4).map(|facing_from| {
                (
                    Vertex {
                        position: *p,
                        facing: facing_from,
                    },
                    array::from_iter_exact(p.nearest_neighbors().enumerate().map(
                        |(facing_to, n)| {
                            if facing_from == facing_to {
                                (map[n] != b'#')
                                    .then(|| {
                                        std::iter::successors(
                                            Some((facing_from, n, *p)),
                                            |(_, current, prev)| {
                                                current
                                                    .nearest_neighbors()
                                                    .enumerate()
                                                    .filter(|(_, nn)| map[*nn] != b'#')
                                                    .find(|(_, nn)| nn != prev)
                                                    .map(|(dir, nn)| (dir, nn, *current))
                                            },
                                        )
                                        .scan(
                                            (facing_from, 0usize, HashSet::new()),
                                            |(dd, dist, seats), (dir, point, _)| {
                                                *dist += if *dd != dir { 1001 } else { 1 };
                                                *dd = dir;
                                                seats.insert(point);
                                                Some(Edge {
                                                    target: Vertex {
                                                        position: point,
                                                        facing: dir,
                                                    },
                                                    distance: *dist,
                                                    seats: seats.clone(),
                                                })
                                            },
                                        )
                                        .find(|edge| junctions.contains(&edge.target.position))
                                    })
                                    .flatten()
                            } else {
                                (map[n] != b'#').then(|| Edge {
                                    target: Vertex {
                                        position: *p,
                                        facing: facing_to,
                                    },
                                    distance: 1000,
                                    seats: HashSet::new(),
                                })
                            }
                        },
                    ))
                    .unwrap(),
                )
            })
        })
        .collect()
}

fn find_shortest_path(graph: &Graph, start: Vector<usize, 2>, end: Vector<usize, 2>) -> usize {
    find_seats(graph, start, end).distance
}

#[derive(Debug, Clone, Default)]
struct SeatState {
    distance: usize,
    seats: HashSet<Vector<usize, 2>>,
}

impl SeatState {
    fn update(mut self, old: Option<&SeatState>) -> Option<SeatState> {
        if let Some(old) = old {
            match self.distance.cmp(&old.distance) {
                std::cmp::Ordering::Less => Some(self),
                std::cmp::Ordering::Equal if self.seats.is_subset(&old.seats) => None,
                std::cmp::Ordering::Equal => {
                    self.seats.extend(old.seats.clone());
                    Some(self)
                }
                std::cmp::Ordering::Greater => None,
            }
        } else {
            Some(self)
        }
    }
}

fn find_seats(graph: &Graph, start: Vector<usize, 2>, end: Vector<usize, 2>) -> SeatState {
    let mut distances = HashMap::from([(
        Vertex {
            position: start,
            facing: 1,
        },
        SeatState::default(),
    )]);

    let mut todo = HashSet::from([Vertex {
        position: start,
        facing: 1,
    }]);
    while let Some(current) = todo.iter().next().cloned() {
        todo.remove(&current);
        for edge in graph.get(&current).unwrap().iter().flatten() {
            let mut target_state = distances.get(&current).unwrap().clone();
            target_state.distance += edge.distance;
            target_state.seats.extend(edge.seats.iter().cloned());

            if let Some(updated) = target_state.update(distances.get(&edge.target)) {
                distances.insert(edge.target.clone(), updated);
                todo.insert(edge.target.clone());
            }
        }
    }

    (0..4)
        .map(|i| Vertex {
            position: end,
            facing: i,
        })
        .filter_map(|end_vertex| distances.get(&end_vertex).cloned())
        .fold(None, |lhs, rhs| rhs.clone().update(lhs.as_ref()).or(lhs))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SMALL_EXAMPLE_INPUT: &str = "\
###############
#.......#....E#
#.#.###.#.###.#
#.....#.#...#.#
#.###.#####.#.#
#.#.#.......#.#
#.#.#####.###.#
#...........#.#
###.#.#####.#.#
#...#.....#.#.#
#.#.#.###.#.#.#
#.....#...#.#.#
#.###.#.#.#.#.#
#S..#.....#...#
###############";
    const SMALL_EXAMPLE_START: Vector<usize, 2> = Vector([13, 1]);
    const SMALL_EXAMPLE_END: Vector<usize, 2> = Vector([1, 13]);

    const LARGER_EXAMPLE_INPUT: &str = "\
#################
#...#...#...#..E#
#.#.#.#.#.#.#.#.#
#.#.#.#...#...#.#
#.#.#.#.###.#.#.#
#...#.#.#.....#.#
#.#.#.#.#.#####.#
#.#...#.#.#.....#
#.#.#####.#.###.#
#.#.#.......#...#
#.#.###.#####.###
#.#.#...#.....#.#
#.#.#.#####.###.#
#.#.#.........#.#
#.#.#.#########.#
#S#.............#
#################";
    const LARGER_EXAMPLE_START: Vector<usize, 2> = Vector([15, 1]);
    const LARGER_EXAMPLE_END: Vector<usize, 2> = Vector([1, 15]);

    #[test]
    fn find_start_and_end() {
        let map = parse_map(SMALL_EXAMPLE_INPUT).unwrap();
        assert_eq!(find_in_map(&map, b'S'), Some(SMALL_EXAMPLE_START));
        assert_eq!(find_in_map(&map, b'E'), Some(SMALL_EXAMPLE_END));
    }

    #[test]
    fn shortest_path_for_small_example() {
        assert_eq!(
            find_shortest_path(
                &small_example_graph(),
                SMALL_EXAMPLE_START,
                SMALL_EXAMPLE_END
            ),
            7036
        );
    }

    #[test]
    fn short_way_up() {
        assert_eq!(
            find_shortest_path(&small_example_graph(), SMALL_EXAMPLE_START, Vector([11, 1])),
            1002
        );
    }

    #[test]
    fn shortest_path_for_larger_example() {
        assert_eq!(
            find_shortest_path(
                &larger_example_graph(),
                LARGER_EXAMPLE_START,
                LARGER_EXAMPLE_END
            ),
            11048
        );
    }

    #[test]
    fn number_of_seats_in_small_example() {
        assert_eq!(
            find_seats(
                &small_example_graph(),
                SMALL_EXAMPLE_START,
                SMALL_EXAMPLE_END
            )
            .seats
            .len()
                + 1,
            45
        );
    }

    #[test]
    fn number_of_seats_in_larger_example() {
        assert_eq!(
            find_seats(
                &larger_example_graph(),
                LARGER_EXAMPLE_START,
                LARGER_EXAMPLE_END
            )
            .seats
            .len()
                + 1,
            64
        );
    }

    fn small_example_graph() -> Graph {
        graph(
            &parse_map(SMALL_EXAMPLE_INPUT).unwrap(),
            SMALL_EXAMPLE_START,
            SMALL_EXAMPLE_END,
        )
    }

    fn larger_example_graph() -> Graph {
        graph(
            &parse_map(LARGER_EXAMPLE_INPUT).unwrap(),
            LARGER_EXAMPLE_START,
            LARGER_EXAMPLE_END,
        )
    }
}
