use aoc_companion::prelude::*;

use itertools::Itertools;

pub(crate) struct Door {
    elves: Vec<Vec<u32>>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Self {
        Self {
            elves: parse_input(input),
        }
    }

    fn part1(&self) -> u32 {
        calories_of_top_n(&self.elves, 1)
    }

    fn part2(&self) -> u32 {
        calories_of_top_n(&self.elves, 3)
    }
}

fn parse_input(input: &str) -> Vec<Vec<u32>> {
    input
        .lines()
        .map(|line| line.parse())
        .chunk_by(|line| line.is_ok())
        .into_iter()
        .filter_map(|(is_number, group)| {
            if is_number {
                Some(group.map(Result::unwrap).collect())
            } else {
                None
            }
        })
        .collect()
}

fn calories_of_top_n<E: AsRef<[u32]>>(elves: &[E], n: usize) -> u32 {
    elves
        .iter()
        .map(|items| items.as_ref().iter().sum())
        .sorted_by(|lhs, rhs| <u32 as Ord>::cmp(rhs, lhs))
        .take(n)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::assert_equal;

    const EXAMPLE_INPUT: &str = r"1000
2000
3000

4000

5000
6000

7000
8000
9000

10000";

    const EXAMPLE_ELVES: &[&[u32]] = &[
        &[1000, 2000, 3000],
        &[4000],
        &[5000, 6000],
        &[7000, 8000, 9000],
        &[10000],
    ];

    #[test]
    fn input_is_parsed() {
        assert_equal(parse_input(EXAMPLE_INPUT).iter(), EXAMPLE_ELVES);
    }

    #[test]
    fn elf_with_most_calories_is_found() {
        assert_eq!(calories_of_top_n(EXAMPLE_ELVES, 1), 24000);
    }

    #[test]
    fn calories_of_top_3_elves() {
        assert_eq!(calories_of_top_n(EXAMPLE_ELVES, 3), 45000);
    }
}
