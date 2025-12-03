use anyhow::Context as _;
use aoc_companion::prelude::*;
use itertools::Itertools as _;

pub(crate) struct Door {
    banks: Vec<Vec<u32>>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        Ok(Door {
            banks: input
                .lines()
                .map(|line| {
                    line.chars()
                        .map(|c| {
                            c.to_digit(10)
                                .with_context(|| anyhow::anyhow!("{c:?} is not a digit"))
                        })
                        .try_collect()
                })
                .try_collect()?,
        })
    }

    fn part1(&self) -> u64 {
        self.banks.iter().map(|b| max_joltage(b, 2)).sum()
    }

    fn part2(&self) -> u64 {
        self.banks.iter().map(|b| max_joltage(b, 12)).sum()
    }
}

fn max_joltage(bank: &[u32], n_battery: usize) -> u64 {
    let (res, _) = (0..n_battery).rev().fold((0, bank), |(acc, available), n| {
        let rev_pos = available.iter().rev().skip(n).position_max().unwrap() + n;
        let max_idx = available.len() - rev_pos - 1;
        let (max, rest) = available[max_idx..].split_first().unwrap();
        (acc * 10 + *max as u64, rest)
    });

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "\
987654321111111
811111111111119
234234234234278
818181911112111";

    const EXAMPLE_BANKS: [[u32; 15]; 4] = [
        [9, 8, 7, 6, 5, 4, 3, 2, 1, 1, 1, 1, 1, 1, 1],
        [8, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 9],
        [2, 3, 4, 2, 3, 4, 2, 3, 4, 2, 3, 4, 2, 7, 8],
        [8, 1, 8, 1, 8, 1, 9, 1, 1, 1, 1, 2, 1, 1, 1],
    ];

    #[test]
    fn parse_example_input() {
        let Door { banks } = Door::parse(EXAMPLE_INPUT).unwrap();
        itertools::assert_equal(banks, EXAMPLE_BANKS);
    }

    #[test]
    fn max_joltage_for_multiple_repeats() {
        assert_eq!(max_joltage(&[9, 9, 1], 2), 99);
    }

    #[test]
    fn example_max_joltages_of_2_batteries() {
        itertools::assert_equal(EXAMPLE_BANKS.map(|b| max_joltage(&b, 2)), [98, 89, 78, 92]);
    }

    #[test]
    fn example_max_joltages_of_12_batteries() {
        itertools::assert_equal(
            EXAMPLE_BANKS.map(|b| max_joltage(&b, 12)),
            [987654321111, 811111111119, 434234234278, 888911112111],
        );
    }
}
