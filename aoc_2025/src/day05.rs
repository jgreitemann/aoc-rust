use std::{collections::HashSet, ops::RangeInclusive};

use anyhow::Context;
use aoc_companion::prelude::*;
use aoc_utils::range::RangeSet;
use itertools::Itertools as _;

pub(crate) struct Door {
    fresh_ranges: Vec<RangeInclusive<u64>>,
    available_ingredients: Vec<u64>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        let Some((fresh, available)) = input.split_once("\n\n") else {
            anyhow::bail!("missing empty line separating fresh ranges from ingredients");
        };
        Ok(Door {
            fresh_ranges: fresh
                .lines()
                .map(|r| -> Result<_> {
                    let (from, to) = r
                        .split_once('-')
                        .with_context(|| anyhow::anyhow!("missing dash in ID range"))?;
                    Ok(from.parse()?..=to.parse()?)
                })
                .try_collect()?,
            available_ingredients: available.lines().map(|i| i.parse()).try_collect()?,
        })
    }

    fn part1(&self) -> usize {
        fresh_ingredients(&self.available_ingredients, &self.fresh_ranges).count()
    }

    fn part2(&self) -> usize {
        disjoint_ranges(&self.fresh_ranges)
            .into_iter()
            .map(|r| r.count())
            .sum()
    }
}

fn fresh_ingredients<'a, I>(
    available_ingredients: impl IntoIterator<Item = &'a u64> + 'a,
    fresh_ranges: I,
) -> impl Iterator<Item = u64> + 'a
where
    I: IntoIterator<Item = &'a RangeInclusive<u64>> + 'a,
    I::IntoIter: Clone,
{
    let fresh_ranges = fresh_ranges.into_iter();
    available_ingredients
        .into_iter()
        .cloned()
        .filter(move |i| fresh_ranges.clone().any(|r| r.contains(i)))
}

fn disjoint_ranges<'a>(
    ranges: impl IntoIterator<Item = &'a RangeInclusive<u64>>,
) -> HashSet<RangeInclusive<u64>> {
    ranges.into_iter().fold(
        HashSet::<RangeInclusive<u64>>::new(),
        |mut disjoint_ranges, r| {
            let overlapping = disjoint_ranges.extract_if(|d| d.try_union(r).is_some());
            let union = overlapping
                .map(|o| o.try_union(r).unwrap())
                .reduce(|lhs, rhs| lhs.try_union(&rhs).unwrap())
                .unwrap_or(r.clone());
            disjoint_ranges.insert(union);
            disjoint_ranges
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "\
3-5
10-14
16-20
12-18

1
5
8
11
17
32";

    const EXAMPLE_FRESH_RANGES: &[RangeInclusive<u64>] = &[3..=5, 10..=14, 16..=20, 12..=18];

    const EXAMPLE_AVAILABLE_INGREDIENTS: &[u64] = &[1, 5, 8, 11, 17, 32];

    #[test]
    fn parse_example_input() {
        let Door {
            fresh_ranges,
            available_ingredients,
        } = Door::parse(EXAMPLE_INPUT).unwrap();
        assert_eq!(fresh_ranges, EXAMPLE_FRESH_RANGES);
        assert_eq!(available_ingredients, EXAMPLE_AVAILABLE_INGREDIENTS);
    }

    #[test]
    fn example_fresh_ingredients() {
        itertools::equal(
            fresh_ingredients(EXAMPLE_AVAILABLE_INGREDIENTS, EXAMPLE_FRESH_RANGES),
            [5, 11, 17],
        );
    }

    #[test]
    fn disjoint_ranges_cover_same_ids() {
        assert_eq!(
            disjoint_ranges(EXAMPLE_FRESH_RANGES)
                .into_iter()
                .flatten()
                .collect::<HashSet<_>>(),
            EXAMPLE_FRESH_RANGES
                .iter()
                .cloned()
                .flatten()
                .collect::<HashSet<_>>()
        );
    }

    #[test]
    fn disjoint_ranges_are_mutually_disjoint() {
        assert_eq!(
            disjoint_ranges(EXAMPLE_FRESH_RANGES)
                .iter()
                .tuple_combinations()
                .find(|(lhs, rhs)| lhs.intersection(rhs).is_some()),
            None
        );
    }
}
