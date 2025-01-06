use aoc_companion::prelude::*;
use aoc_utils::geometry::try_parse_map;

pub(crate) struct Door {
    map: Map,
}

type Map = ndarray::Array2<bool>;

#[derive(Debug, thiserror::Error)]
#[error("invalid tile")]
struct InvalidTile;

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        Ok(Door {
            map: try_parse_map(input, |b| match b {
                b'.' => Ok(false),
                b'#' => Ok(true),
                _ => Err(InvalidTile),
            })?,
        })
    }

    fn part1(&self) -> usize {
        trees_on_slope(Slope { right: 3, down: 1 }, &self.map)
    }

    fn part2(&self) -> usize {
        [
            Slope { right: 1, down: 1 },
            Slope { right: 3, down: 1 },
            Slope { right: 5, down: 1 },
            Slope { right: 7, down: 1 },
            Slope { right: 1, down: 2 },
        ]
        .map(|slope| trees_on_slope(slope, &self.map))
        .iter()
        .product()
    }
}

#[derive(Copy, Clone)]
struct Slope {
    right: usize,
    down: usize,
}

trait SlopeIterable {
    fn slope_iter(&self, slope: Slope) -> SlopeIter;
}

impl SlopeIterable for Map {
    fn slope_iter(&self, slope: Slope) -> SlopeIter {
        SlopeIter {
            game_board: self,
            row_iter: Box::new((0..self.nrows()).step_by(slope.down)),
            col_iter: Box::new((0..self.ncols()).cycle().step_by(slope.right)),
        }
    }
}

struct SlopeIter<'a> {
    game_board: &'a Map,
    row_iter: Box<dyn Iterator<Item = usize>>,
    col_iter: Box<dyn Iterator<Item = usize>>,
}

impl Iterator for SlopeIter<'_> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(row_idx) = self.row_iter.next() {
            let col_idx = self.col_iter.next().unwrap();

            Some(self.game_board[(row_idx, col_idx)])
        } else {
            None
        }
    }
}

fn trees_on_slope(slope: Slope, map: &Map) -> usize {
    map.slope_iter(slope).filter(|&x| x).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "\
..##.......
#...#...#..
.#....#..#.
..#.#...#.#
.#...##..#.
..#.##.....
.#.#.#....#
.#........#
#.##...#...
#...##....#
.#..#...#.#";

    #[test]
    fn trees_on_slopes_in_example() {
        let Door { map } = Door::parse(EXAMPLE_INPUT).unwrap();
        assert_eq!(trees_on_slope(Slope { right: 1, down: 1 }, &map), 2);
        assert_eq!(trees_on_slope(Slope { right: 3, down: 1 }, &map), 7);
        assert_eq!(trees_on_slope(Slope { right: 5, down: 1 }, &map), 3);
        assert_eq!(trees_on_slope(Slope { right: 7, down: 1 }, &map), 4);
        assert_eq!(trees_on_slope(Slope { right: 1, down: 2 }, &map), 2);
    }
}
