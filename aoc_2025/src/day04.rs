use std::collections::HashSet;

use aoc_companion::prelude::*;
use aoc_utils::geometry::{ParseMapError, Point};
use aoc_utils::linalg::Vector;
use itertools::Itertools;

pub(crate) struct Door {
    rolls: HashSet<Vector<usize, 2>>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseMapError<ParseTileError>> {
        Ok(Door {
            rolls: parse_roll_locations(input)?,
        })
    }

    fn part1(&self) -> usize {
        accessible_roll_locations(&self.rolls).count()
    }

    fn part2(&self) -> usize {
        self.rolls.len() - non_removable_roll_locations(self.rolls.clone()).len()
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ParseTileError {
    #[error("invalid map tile: {tile:?}")]
    InvalidTile { tile: u8 },
}

fn parse_roll_locations(
    input: &str,
) -> Result<HashSet<Vector<usize, 2>>, ParseMapError<ParseTileError>> {
    let map = aoc_utils::geometry::try_parse_map(input, |tile| match tile {
        b'@' => Ok(true),
        b'.' => Ok(false),
        _ => Err(ParseTileError::InvalidTile { tile }),
    })?;

    Ok(map
        .indexed_iter()
        .filter(|(_, is_roll)| **is_roll)
        .map(|((y, x), _)| Vector([x, y]))
        .collect())
}

fn accessible_roll_locations(
    rolls: &HashSet<Vector<usize, 2>>,
) -> impl Iterator<Item = Vector<usize, 2>> {
    rolls
        .iter()
        .filter(|r| r.neighbors().filter(|n| rolls.contains(n)).count() < 4)
        .copied()
}

fn with_accessible_rolls_removed(rolls: &HashSet<Vector<usize, 2>>) -> HashSet<Vector<usize, 2>> {
    rolls
        .difference(&accessible_roll_locations(rolls).collect())
        .copied()
        .collect()
}

fn non_removable_roll_locations(rolls: HashSet<Vector<usize, 2>>) -> HashSet<Vector<usize, 2>> {
    std::iter::successors(Some(rolls), |prev| {
        Some(with_accessible_rolls_removed(prev))
    })
    .tuple_windows()
    .find_map(|(lhs, rhs)| (lhs == rhs).then_some(lhs))
    .unwrap()
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    const EXAMPLE_INPUT: &str = "\
..@@.@@@@.
@@@.@.@.@@
@@@@@.@.@@
@.@@@@..@.
@@.@@@@.@@
.@@@@@@@.@
.@.@.@.@@@
@.@@@.@@@@
.@@@@@@@@.
@.@.@@@.@.";

    const EXAMPLE_ACCESSIBLE_ROLLS: &str = "\
..@@.@@.@.
@.........
......@...
..........
@........@
..........
..........
@.........
..........
@.@.....@.";

    const EXAMPLE_NON_REMOVABLE_ROLLS: &str = "\
..........
..........
..........
....@@....
...@@@@...
...@@@@@..
...@.@.@@.
...@@.@@@.
...@@@@@..
....@@@...";

    const EXAMPLE_ROLL_LOCATIONS: &[Vector<usize, 2>] = &[
        Vector([0, 1]),
        Vector([0, 2]),
        Vector([0, 3]),
        Vector([0, 4]),
        Vector([0, 7]),
        Vector([0, 9]),
        Vector([1, 1]),
        Vector([1, 2]),
        Vector([1, 4]),
        Vector([1, 5]),
        Vector([1, 6]),
        Vector([1, 8]),
        Vector([2, 0]),
        Vector([2, 1]),
        Vector([2, 2]),
        Vector([2, 3]),
        Vector([2, 5]),
        Vector([2, 7]),
        Vector([2, 8]),
        Vector([2, 9]),
        Vector([3, 0]),
        Vector([3, 2]),
        Vector([3, 3]),
        Vector([3, 4]),
        Vector([3, 5]),
        Vector([3, 6]),
        Vector([3, 7]),
        Vector([3, 8]),
        Vector([4, 1]),
        Vector([4, 2]),
        Vector([4, 3]),
        Vector([4, 4]),
        Vector([4, 5]),
        Vector([4, 7]),
        Vector([4, 8]),
        Vector([4, 9]),
        Vector([5, 0]),
        Vector([5, 3]),
        Vector([5, 4]),
        Vector([5, 5]),
        Vector([5, 6]),
        Vector([5, 8]),
        Vector([5, 9]),
        Vector([6, 0]),
        Vector([6, 1]),
        Vector([6, 2]),
        Vector([6, 4]),
        Vector([6, 5]),
        Vector([6, 7]),
        Vector([6, 8]),
        Vector([6, 9]),
        Vector([7, 0]),
        Vector([7, 5]),
        Vector([7, 6]),
        Vector([7, 7]),
        Vector([7, 8]),
        Vector([8, 0]),
        Vector([8, 1]),
        Vector([8, 2]),
        Vector([8, 3]),
        Vector([8, 4]),
        Vector([8, 6]),
        Vector([8, 7]),
        Vector([8, 8]),
        Vector([8, 9]),
        Vector([9, 1]),
        Vector([9, 2]),
        Vector([9, 4]),
        Vector([9, 5]),
        Vector([9, 6]),
        Vector([9, 7]),
    ];

    #[test]
    fn parse_example_roll_locations() {
        itertools::assert_equal(
            parse_roll_locations(EXAMPLE_INPUT).unwrap().iter().sorted(),
            EXAMPLE_ROLL_LOCATIONS,
        );
    }

    #[test]
    fn accessible_rolls_in_example() {
        assert_eq!(
            HashSet::from_iter(accessible_roll_locations(
                &parse_roll_locations(EXAMPLE_INPUT).unwrap()
            )),
            parse_roll_locations(EXAMPLE_ACCESSIBLE_ROLLS).unwrap()
        );
    }

    #[test]
    fn non_removable_rolls_in_example() {
        assert_eq!(
            non_removable_roll_locations(parse_roll_locations(EXAMPLE_INPUT).unwrap()),
            parse_roll_locations(EXAMPLE_NON_REMOVABLE_ROLLS).unwrap()
        )
    }
}
