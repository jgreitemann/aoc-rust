use std::{collections::HashMap, rc::Rc};

use anyhow::bail;
use aoc_companion::prelude::*;
use aoc_utils::iter::{IterUtils, RcIter};
use itertools::Itertools;

pub(crate) struct Door {
    components: HashMap<(u8, u8), usize>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        Ok(Door {
            components: parse_components(input)?,
        })
    }

    fn part1(&self) -> usize {
        find_path_max_strength(&self.components)
    }

    fn part2(&self) -> impl door::IntoResult {
        find_strength_of_max_length(&self.components)
    }
}

fn parse_components(input: &str) -> Result<HashMap<(u8, u8), usize>> {
    let components: Vec<_> = input
        .lines()
        .map(|line| -> Result<(u8, u8)> {
            let Some((lhs, rhs)) = line.split_once('/') else {
                bail!("Missing slash in component line");
            };
            let mut lhs = lhs.parse()?;
            let mut rhs = rhs.parse()?;
            if lhs > rhs {
                std::mem::swap(&mut lhs, &mut rhs);
            }
            Ok((lhs, rhs))
        })
        .try_collect()?;

    Ok(components.into_iter().counts())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Path {
    remaining: HashMap<(u8, u8), usize>,
    tail: u8,
}

fn path_iter(components: &HashMap<(u8, u8), usize>) -> impl Iterator<Item = Path> {
    let lookup = compute_lookup(components);
    path_dfs(
        Path {
            remaining: components.clone(),
            tail: 0,
        },
        Rc::new(lookup),
    )
}

fn path_dfs<'a>(
    path: Path,
    lookup: Rc<HashMap<u8, Rc<[u8]>>>,
) -> Box<dyn Iterator<Item = Path> + 'a> {
    let Path { remaining, tail } = path.clone();
    let candidates = RcIter::new(lookup[&tail].clone());

    #[allow(clippy::filter_map_bool_then)]
    Box::new(
        candidates
            .map(move |rhs| if tail < rhs { (tail, rhs) } else { (rhs, tail) })
            .filter_map(move |candidate| {
                (remaining[&candidate] > 0).then(|| {
                    let mut remaining = remaining.clone();
                    *remaining.get_mut(&candidate).unwrap() -= 1;
                    let (lhs, rhs) = candidate;
                    let tail = if lhs == path.tail { rhs } else { lhs };
                    Path { remaining, tail }
                })
            })
            .flat_map(move |new_path| path_dfs(new_path, lookup.clone()))
            .fallback(path),
    )
}

fn compute_lookup(components: &HashMap<(u8, u8), usize>) -> HashMap<u8, Rc<[u8]>> {
    components
        .keys()
        .flat_map(|&(lhs, rhs)| [(lhs, rhs), (rhs, lhs)])
        .into_group_map()
        .into_iter()
        .map(|(k, vs)| (k, vs.into()))
        .collect()
}

fn strength(components: &HashMap<(u8, u8), usize>) -> usize {
    components
        .iter()
        .map(|(&(lhs, rhs), &count)| count * (lhs as usize + rhs as usize))
        .sum()
}

fn length(components: &HashMap<(u8, u8), usize>) -> usize {
    components.values().copied().sum()
}

fn find_path_max_strength(components: &HashMap<(u8, u8), usize>) -> usize {
    let full_strength = strength(components);
    let min_complement_strength = path_iter(components)
        .map(|path| strength(&path.remaining))
        .min()
        .unwrap();
    full_strength - min_complement_strength
}

fn find_strength_of_max_length(components: &HashMap<(u8, u8), usize>) -> usize {
    let full_strength = strength(components);
    let (_min_complement_length, min_complement_strength) = path_iter(components)
        .map(|path| (length(&path.remaining), strength(&path.remaining)))
        .min()
        .unwrap();
    full_strength - min_complement_strength
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "\
0/2
2/2
2/3
3/4
3/5
0/1
10/1
9/10";

    const EXAMPLE_COMPONENTS: [((u8, u8), usize); 8] = [
        ((0, 2), 1),
        ((2, 2), 1),
        ((2, 3), 1),
        ((3, 4), 1),
        ((3, 5), 1),
        ((0, 1), 1),
        ((1, 10), 1),
        ((9, 10), 1),
    ];

    #[test]
    fn parse_example_input() {
        assert_eq!(
            parse_components(EXAMPLE_INPUT).unwrap(),
            HashMap::from(EXAMPLE_COMPONENTS)
        );
    }

    #[test]
    fn example_max_strength_path() {
        assert_eq!(
            find_path_max_strength(&HashMap::from(EXAMPLE_COMPONENTS)),
            31
        );
    }

    #[test]
    fn example_max_strength_of_max_length_path() {
        assert_eq!(
            find_strength_of_max_length(&HashMap::from(EXAMPLE_COMPONENTS)),
            19
        );
    }
}
