use std::{collections::HashSet, num::ParseIntError};

use anyhow::{Context, anyhow, bail};
use aoc_companion::prelude::*;
use itertools::Itertools as _;

pub(crate) struct Door {
    problems: Vec<Problem>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        let Some((shapes, problems)) = input.rsplit_once("\n\n") else {
            bail!("could not find empty line delimiting shapes and problems");
        };
        let shapes_hash = fxhash::hash(shapes);
        if shapes_hash != 0x2050bd894f01430f {
            bail!("solution only works for my input; got different hash: {shapes_hash:x}");
        }
        problems
            .lines()
            .map(str::parse)
            .try_collect()
            .map(|problems| Door { problems })
    }

    fn part1(&self) -> Result<usize> {
        let (conclusive, inconclusive): (Vec<bool>, HashSet<&Problem>) = self
            .problems
            .iter()
            .map(|problem| {
                rule_out_due_to_insufficient_area(problem)
                    .or_else(|| verify_with_trivial_packing(problem))
                    .ok_or(problem)
            })
            .partition_result();

        if !inconclusive.is_empty() {
            bail!(
                "{}/{} problems were inconclusive, e.g.: {}",
                inconclusive.len(),
                self.problems.len(),
                inconclusive.iter().next().unwrap()
            );
        }

        Ok(conclusive.into_iter().filter(|&b| b).count())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Problem {
    dimensions: [usize; 2],
    presents: [usize; 6],
}

impl std::fmt::Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}",
            self.dimensions.iter().join("x"),
            self.presents.iter().join(" ")
        )
    }
}

impl std::str::FromStr for Problem {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let Some((dimensions, shapes)) = s.split_once(':') else {
            bail!("could not find colon delimiting problem dimensions and required shapes");
        };
        let dimensions = dimensions
            .split_once('x')
            .ok_or_else(|| anyhow!("missing 'x' in dimensions spec {dimensions:?}"))
            .and_then(|(x, y)| {
                Ok::<_, ParseIntError>([x.parse()?, y.parse()?])
                    .with_context(|| anyhow!("failed to parse region dimenions {dimensions:?}"))
            })?;
        // TODO: add try_from_iter_exact
        let presents = aoc_utils::array::try_from_iter(shapes.split_whitespace().map(|s| {
            s.parse()
                .with_context(|| anyhow!("failed to parse present count {s:?}"))
        }))?
        .map_err(|e| anyhow!("wrong number of presents: expected 6, got {}", e.len()))?;

        Ok(Self {
            dimensions,
            presents,
        })
    }
}

impl Problem {
    fn area(&self) -> usize {
        self.dimensions[0] * self.dimensions[1]
    }
}

fn rule_out_due_to_insufficient_area(problem: &Problem) -> Option<bool> {
    const AREA: [usize; 6] = [5, 7, 7, 7, 6, 7];

    let required_area: usize = problem
        .presents
        .iter()
        .zip(AREA)
        .map(|(count, area)| count * area)
        .sum();

    (required_area > problem.area()).then_some(false)
}

fn verify_with_trivial_packing(problem: &Problem) -> Option<bool> {
    let available_cells = (problem.dimensions[0] / 3) * (problem.dimensions[1] / 3);

    let total_presents: usize = problem.presents.iter().sum();

    (total_presents <= available_cells).then_some(true)
}
