use aoc_companion::prelude::*;
use aoc_utils::{geometry::Point, linalg::Vector};

use itertools::Itertools;
use ndarray::Array2;
use thiserror::Error;

use std::collections::VecDeque;
use std::str::FromStr;

pub struct Door {
    map: Map,
}

impl ParseInput<'_> for Door {
    type Error = ParseError;

    fn parse(input: &str) -> Result<Self, Self::Error> {
        input.parse().map(|map| Self { map })
    }
}

impl Part1 for Door {
    type Output = usize;
    type Error = std::convert::Infallible;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        let flow = self.map.dijkstra_flow();
        Ok(flow[self.map.start])
    }
}

impl Part2 for Door {
    type Output = usize;
    type Error = std::convert::Infallible;

    fn part2(&self) -> Result<Self::Output, Self::Error> {
        let flow = self.map.dijkstra_flow();
        Ok(self
            .map
            .points_with_elevation(0)
            .map(|p| flow[p])
            .min()
            .unwrap())
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Could not find starting position (S) on map")]
    StartMissing,
    #[error("Could not find destination position (E) on map")]
    EndMissing,
    #[error("Map is not rectangular: {0}")]
    ShapeError(#[from] ndarray::ShapeError),
}

struct Map {
    elevations: Array2<u8>,
    start: Vector<usize, 2>,
    end: Vector<usize, 2>,
}

fn neighbors_checked(
    p: Vector<usize, 2>,
    shape: &[usize],
) -> impl Iterator<Item = Vector<usize, 2>> + '_ {
    let q = Vector([p[0] as i64, p[1] as i64]);
    q.nearest_neighbors()
        .filter(|n| (0..shape[0] as i64).contains(&n[0]) && (0..shape[1] as i64).contains(&n[1]))
        .map(|n| Vector([n[0] as usize, n[1] as usize]))
}

impl Map {
    fn dijkstra_flow(&self) -> Array2<usize> {
        let mut flow = Array2::from_elem(self.elevations.raw_dim(), usize::MAX);
        flow[self.end] = 0;

        let mut active = VecDeque::from([self.end]);
        while let Some(p) = active.pop_front() {
            let distance = flow[p] + 1;
            for n in neighbors_checked(p, self.elevations.shape()) {
                if self.elevations[n] + 1 >= self.elevations[p] {
                    if flow[n] > distance {
                        flow[n] = distance;
                        active.push_back(n);
                    }
                }
            }
        }

        flow
    }

    fn points_with_elevation(
        &self,
        target_elevation: u8,
    ) -> impl Iterator<Item = Vector<usize, 2>> + '_ {
        self.elevations
            .indexed_iter()
            .filter_map(move |elem| match elem {
                ((x, y), ele) if *ele == target_elevation => Some(Vector([x, y])),
                _ => None,
            })
    }
}

fn find_byte_coords<R: AsRef<[u8]>>(bytes: &[R], target: u8) -> Option<Vector<usize, 2>> {
    if let Some((x, Some(y))) = bytes
        .iter()
        .map(|row| row.as_ref().iter().position(|&c| c == target))
        .find_position(|opt| opt.is_some())
    {
        Some(Vector([x, y]))
    } else {
        None
    }
}

impl FromStr for Map {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut bytes: Vec<_> = s.lines().map(|line| line.as_bytes().to_vec()).collect();
        let start = find_byte_coords(&bytes, b'S').ok_or(ParseError::StartMissing)?;
        let end = find_byte_coords(&bytes, b'E').ok_or(ParseError::EndMissing)?;
        bytes[start[0]][start[1]] = b'a';
        bytes[end[0]][end[1]] = b'z';
        let shape = (bytes.len(), bytes.first().unwrap().len());
        let bytes = bytes.into_iter().flatten().map(|c| c - b'a').collect();
        let elevations = Array2::<u8>::from_shape_vec(shape, bytes)?;
        Ok(Self {
            elevations,
            start,
            end,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "Sabqponm\n\
                                 abcryxxl\n\
                                 accszExk\n\
                                 acctuvwj\n\
                                 abdefghi";

    #[test]
    fn map_is_parsed() {
        let map: Map = EXAMPLE_INPUT.parse().unwrap();
        assert_eq!(map.elevations.shape(), [5, 8]);
        assert_eq!(map.start, Vector([0, 0]));
        assert_eq!(map.end, Vector([2, 5]));
        assert_eq!(map.elevations[map.start], 0);
        assert_eq!(map.elevations[map.end], 25);
        assert_eq!(map.elevations[(1, 0)], 0);
        assert_eq!(map.elevations[(0, 1)], 0);
        assert_eq!(map.elevations[(0, 2)], 1);
        assert_eq!(map.elevations[(2, 6)], 23);
        assert_eq!(map.elevations[(4, 7)], 8);
    }

    #[test]
    fn dijkstra_flow_yields_correct_number_of_steps() {
        let map: Map = EXAMPLE_INPUT.parse().unwrap();
        let flow = map.dijkstra_flow();
        assert_eq!(flow[map.start], 31);
    }

    #[test]
    fn shortest_hiking_trail() {
        let map: Map = EXAMPLE_INPUT.parse().unwrap();
        let flow = map.dijkstra_flow();
        assert_eq!(
            map.points_with_elevation(0).map(|p| flow[p]).min(),
            Some(29)
        );
    }
}
