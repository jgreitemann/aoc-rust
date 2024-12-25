use aoc_companion::prelude::*;
use aoc_utils::array;
use itertools::{Either, Itertools};

pub(crate) struct Door {
    locks: Vec<[usize; 5]>,
    keys: Vec<[usize; 5]>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Self {
        let (locks, keys) = input.split("\n\n").partition_map(|block| {
            let block: Vec<[u8; 5]> = block
                .lines()
                .map(|line| array::from_iter_exact(line.as_bytes().iter().copied()).unwrap())
                .collect_vec();
            let item = array::from_iter_exact(
                (0..5).map(|col| (1..6).filter(|&row| block[row][col] == b'#').count()),
            )
            .unwrap();
            if block[0][0] == b'#' {
                Either::Left(item)
            } else {
                Either::Right(item)
            }
        });

        Door { locks, keys }
    }

    fn part1(&self) -> usize {
        non_overlapping_pairs(&self.locks, &self.keys).count()
    }
}

fn does_not_overlap(lock: &[usize; 5], key: &[usize; 5]) -> bool {
    lock.iter().zip(key).all(|(l, k)| l + k <= 5)
}

fn non_overlapping_pairs<'l, 'k>(
    locks: &'l [[usize; 5]],
    keys: &'k [[usize; 5]],
) -> impl Iterator<Item = (&'l [usize; 5], &'k [usize; 5])> + use<'l, 'k> {
    locks
        .iter()
        .cartesian_product(keys)
        .filter(|(lock, key)| does_not_overlap(lock, key))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "\
#####
.####
.####
.####
.#.#.
.#...
.....

#####
##.##
.#.##
...##
...#.
...#.
.....

.....
#....
#....
#...#
#.#.#
#.###
#####

.....
.....
#.#..
###..
###.#
###.#
#####

.....
.....
.....
#....
#.#..
#.#.#
#####";

    const EXAMPLE_LOCKS: &[[usize; 5]] = &[[0, 5, 3, 4, 3], [1, 2, 0, 5, 3]];

    const EXAMPLE_KEYS: &[[usize; 5]] = &[[5, 0, 2, 1, 3], [4, 3, 4, 0, 2], [3, 0, 2, 0, 1]];

    #[test]
    fn parse_example_input() {
        let door = Door::parse(EXAMPLE_INPUT);
        assert_eq!(door.locks, EXAMPLE_LOCKS);
        assert_eq!(door.keys, EXAMPLE_KEYS);
    }

    #[test]
    fn non_overlapping_example_pairs() {
        assert_eq!(
            non_overlapping_pairs(EXAMPLE_LOCKS, EXAMPLE_KEYS).collect_vec(),
            [
                (&[0, 5, 3, 4, 3], &[3, 0, 2, 0, 1]),
                (&[1, 2, 0, 5, 3], &[4, 3, 4, 0, 2]),
                (&[1, 2, 0, 5, 3], &[3, 0, 2, 0, 1]),
            ]
        );
    }
}
