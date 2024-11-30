use std::collections::HashSet;

use aoc_companion::prelude::*;
use aoc_utils::geometry::Point;
use aoc_utils::linalg::Vector;
use itertools::Itertools;

use crate::day10::KnotHash;

pub(crate) struct Door {
    rows: [KnotHash; 128],
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Self {
        Door {
            rows: std::array::from_fn(|row| KnotHash::hash(&format!("{input}-{row}"))),
        }
    }

    fn part1(&self) -> u32 {
        self.count_ones()
    }

    fn part2(&self) -> usize {
        self.number_of_regions()
    }
}

impl Door {
    fn count_ones(&self) -> u32 {
        self.rows.iter().copied().map(KnotHash::count_ones).sum()
    }

    fn contains(&self, Vector([col, row]): Vector<u8, 2>) -> bool {
        let byte = (col >> 3) as usize;
        let bit = 7 - (col & 0b00000111);
        self.rows[row as usize].0[byte] & (1 << bit) != 0
    }

    fn number_of_regions(&self) -> usize {
        let grid_points = (0..128)
            .cartesian_product(0..128)
            .map(|(col, row)| Vector([col, row]));
        let mut visited = HashSet::new();
        grid_points
            .filter(|&p| self.contains(p))
            .filter(|&p| {
                let new_region = !visited.contains(&p);
                if new_region {
                    let mut queue = vec![p];
                    while let Some(q) = queue.pop() {
                        if self.contains(q) && visited.insert(q) {
                            queue.extend(
                                q.nearest_neighbors()
                                    .filter(|&Vector([x, y])| x < 128 && y < 128),
                            );
                        }
                    }
                }
                new_region
            })
            .count()
    }
}

impl KnotHash {
    pub fn count_ones(self) -> u32 {
        self.0.iter().copied().map(u8::count_ones).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_used_squares() {
        assert_eq!(Door::parse("flqrgnkx").count_ones(), 8108);
    }

    #[test]
    fn count_number_of_regions() {
        assert_eq!(Door::parse("flqrgnkx").number_of_regions(), 1242);
    }

    #[test]
    fn indexing_into_grid() {
        let door = Door::parse("flqrgnkx");
        let rows: Vec<String> = (0..8)
            .map(|row| {
                (0..8)
                    .map(|col| {
                        if door.contains(Vector([col, row])) {
                            "#"
                        } else {
                            "."
                        }
                    })
                    .join("")
            })
            .collect();
        assert_eq!(
            rows,
            [
                "##.#.#..", ".#.#.#.#", "....#.#.", "#.#.##.#", ".##.#...", "##..#..#", ".#...#..",
                "##.#.##.",
            ]
        );
    }
}
