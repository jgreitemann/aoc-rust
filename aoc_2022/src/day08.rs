use aoc_companion::prelude::*;

use itertools::Itertools;
use std::str::FromStr;
use thiserror::Error;

pub(crate) struct Door {
    map: Map,
}

#[derive(Debug, Error)]
pub(crate) enum ParseError {
    #[error("The input is empty")]
    EmptyInput,
    #[error("Encountered a non-numeric character: {0:?}")]
    NonNumericCharacter(char),
    #[error("The map's shape is not rectangular: {0}")]
    NonRectangularShape(#[from] ndarray::ShapeError),
}

impl<'input> ParseInput<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseError> {
        Ok(Self {
            map: input.parse()?,
        })
    }
}

impl Part1 for Door {
    fn part1(&self) -> usize {
        self.map.visible_tree_count()
    }
}

impl Part2 for Door {
    fn part2(&self) -> usize {
        self.map.max_scenic_score()
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Map(ndarray::Array2<i32>);

impl FromStr for Map {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let w = s.lines().next().ok_or(ParseError::EmptyInput)?.len();
        let h = s.lines().count();
        let data = s
            .lines()
            .flat_map(str::chars)
            .map(|c| c.to_digit(10).ok_or(ParseError::NonNumericCharacter(c)))
            .map_ok(|u| u as i32)
            .try_collect()?;
        Ok(Self(ndarray::Array::from_shape_vec((w, h), data)?))
    }
}

fn apply_vis<'h, 'v>(
    height_iter: impl Iterator<Item = &'h i32>,
    vis_iter: impl Iterator<Item = &'v mut usize>,
) {
    height_iter.zip(vis_iter).fold(-1i32, |max_height, (x, v)| {
        *v += (*x > max_height) as usize;
        max_height.max(*x)
    });
}

fn apply_vis_both_ways<'h, 'v>(
    height_lanes: impl IntoIterator<Item = ndarray::ArrayView1<'h, i32>>,
    vis_lanes: impl IntoIterator<Item = ndarray::ArrayViewMut1<'v, usize>>,
) {
    for (r, mut v) in height_lanes.into_iter().zip(vis_lanes) {
        apply_vis(r.into_iter(), v.iter_mut());
        apply_vis(r.into_iter().rev(), v.iter_mut().rev());
    }
}

impl Map {
    fn visible_tree_count(&self) -> usize {
        let mut visibilities = ndarray::Array2::<usize>::zeros(self.0.raw_dim());

        apply_vis_both_ways(self.0.rows(), visibilities.rows_mut());
        apply_vis_both_ways(self.0.columns(), visibilities.columns_mut());

        visibilities.into_iter().filter(|&v| v > 0).count()
    }

    fn scenic_score_at(&self, (x, y): (usize, usize)) -> usize {
        viewing_distance(self.0.row(y).into_iter().skip(x))
            * viewing_distance(self.0.row(y).into_iter().take(x + 1).rev())
            * viewing_distance(self.0.column(x).into_iter().skip(y))
            * viewing_distance(self.0.column(x).into_iter().take(y + 1).rev())
    }

    fn max_scenic_score(&self) -> usize {
        ndarray::indices_of(&self.0)
            .into_iter()
            .map(|idx| self.scenic_score_at(idx))
            .max()
            .unwrap()
    }
}

fn viewing_distance<'a>(mut iter: impl Iterator<Item = &'a i32>) -> usize {
    let height = iter.next().unwrap();
    iter.scan(true, |ok, x| {
        std::mem::replace(ok, x < height).then_some(())
    })
    .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = r"30373
25512
65332
33549
35390";

    fn example_map() -> Map {
        Map(ndarray::array![
            [3, 0, 3, 7, 3],
            [2, 5, 5, 1, 2],
            [6, 5, 3, 3, 2],
            [3, 3, 5, 4, 9],
            [3, 5, 3, 9, 0],
        ])
    }

    #[test]
    fn input_map_is_parsed() {
        assert_eq!(EXAMPLE_INPUT.parse::<Map>().unwrap(), example_map());
    }

    #[test]
    fn count_visible_trees() {
        assert_eq!(example_map().visible_tree_count(), 21);
    }

    #[test]
    fn calculate_scenic_score() {
        assert_eq!(example_map().scenic_score_at((2, 1)), 4);
    }

    #[test]
    fn find_max_scenic_score() {
        assert_eq!(example_map().max_scenic_score(), 8);
    }
}
