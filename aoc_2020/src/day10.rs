use aoc_companion::prelude::*;
use itertools::Itertools;

use std::iter::{once, zip};
use std::num::ParseIntError;

pub(crate) struct Door {
    levels: Vec<u32>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseIntError> {
        let mut levels: Vec<_> = input.lines().map(str::parse).try_collect()?;
        levels.push(0);
        levels.sort();
        levels.push(levels.last().unwrap() + 3);
        Ok(Door { levels })
    }

    fn part1(&self) -> usize {
        let (diff1, _, diff3) = joltage_jump_statistics(&self.levels);
        diff1 * diff3
    }

    fn part2(&self) -> usize {
        number_of_valid_combos(&self.levels)
    }
}

fn joltage_jumps<'a, I>(levels_iter: I) -> impl Iterator<Item = u32> + 'a
where
    I: Iterator<Item = &'a u32> + 'a,
{
    levels_iter
        .tuple_windows()
        .map(|(lower, higher)| higher - lower)
}

fn joltage_jump_statistics(levels: &[u32]) -> (usize, usize, usize) {
    let freq = joltage_jumps(levels.iter()).counts();

    (
        freq.get(&1).copied().unwrap_or(0),
        freq.get(&2).copied().unwrap_or(0),
        freq.get(&3).copied().unwrap_or(0),
    )
}

fn is_valid_combo<'a, I>(levels_iter: I) -> bool
where
    I: Iterator<Item = &'a u32> + 'a,
{
    joltage_jumps(levels_iter).all(|jump| (1..=3).contains(&jump))
}

fn group_levels_by_max_jump(levels: &[u32]) -> Vec<Vec<u32>> {
    zip(levels.iter(), joltage_jumps(levels.iter()).chain(once(0)))
        .collect_vec()
        .split_inclusive(|(_, jmp)| *jmp == 3)
        .map(|group| group.iter().map(|&(&lvl, _)| lvl).collect_vec())
        .collect_vec()
}

fn number_of_valid_combos(levels: &[u32]) -> usize {
    group_levels_by_max_jump(levels)
        .iter()
        .map(|g| number_of_valid_combos_small(g))
        .reduce(|acc, next| acc * next)
        .unwrap()
}

fn number_of_valid_combos_small(levels: &[u32]) -> usize {
    if levels.len() < 2 {
        return 1;
    }

    let (device_joltage, rest) = levels.split_last().unwrap();
    let (outlet_joltage, adapters) = rest.split_first().unwrap();
    adapters
        .iter()
        .copied()
        .powerset()
        .filter(|subset| {
            let subset_levels = Iterator::chain(
                once(outlet_joltage),
                subset.iter().chain(once(device_joltage)),
            );
            is_valid_combo(subset_levels)
        })
        .count()
}

#[cfg(test)]
mod test {
    const SMALL_EXAMPLE_LEVELS: [u32; 13] = [0, 1, 4, 5, 6, 7, 10, 11, 12, 15, 16, 19, 22];
    const SMALL_EXAMPLE_GROUPS: [&[u32]; 6] = [
        &[0, 1],
        &[4, 5, 6, 7],
        &[10, 11, 12],
        &[15, 16],
        &[19],
        &[22],
    ];

    const LARGE_EXAMPLE_LEVELS: [u32; 33] = [
        0, 1, 2, 3, 4, 7, 8, 9, 10, 11, 14, 17, 18, 19, 20, 23, 24, 25, 28, 31, 32, 33, 34, 35, 38,
        39, 42, 45, 46, 47, 48, 49, 52,
    ];

    #[test]
    fn joltage_jump_statistics() {
        use super::joltage_jump_statistics;
        assert_eq!(joltage_jump_statistics(&SMALL_EXAMPLE_LEVELS), (7, 0, 5));
        assert_eq!(joltage_jump_statistics(&LARGE_EXAMPLE_LEVELS), (22, 0, 10));
    }

    #[test]
    fn group_levels_by_max_jump() {
        let groups = SMALL_EXAMPLE_GROUPS.map(|g| g.to_vec());
        itertools::assert_equal(
            super::group_levels_by_max_jump(&SMALL_EXAMPLE_LEVELS),
            groups,
        );
    }

    #[test]
    fn number_of_valid_combos_small() {
        assert_eq!(
            super::number_of_valid_combos_small(&SMALL_EXAMPLE_LEVELS),
            8
        );
        assert_eq!(
            super::number_of_valid_combos_small(SMALL_EXAMPLE_GROUPS[1]),
            4
        );
        assert_eq!(
            super::number_of_valid_combos_small(SMALL_EXAMPLE_GROUPS[2]),
            2
        );
        assert_eq!(
            super::number_of_valid_combos_small(SMALL_EXAMPLE_GROUPS[3]),
            1
        );
        assert_eq!(
            super::number_of_valid_combos_small(SMALL_EXAMPLE_GROUPS[4]),
            1
        );
    }

    #[test]
    fn number_of_valid_combos() {
        assert_eq!(super::number_of_valid_combos(&SMALL_EXAMPLE_LEVELS), 8);
        assert_eq!(super::number_of_valid_combos(&LARGE_EXAMPLE_LEVELS), 19208);
    }
}
