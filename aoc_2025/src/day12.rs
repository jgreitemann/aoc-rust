use std::{collections::HashSet, num::ParseIntError};

use anyhow::{Context, anyhow, bail};
use aoc_companion::prelude::*;
use itertools::Itertools as _;

pub(crate) struct Door {
    areas: [usize; 6],
    problems: Vec<Problem>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        let Some((shapes, problems)) = input.rsplit_once("\n\n") else {
            bail!("could not find empty line delimiting shapes and problems");
        };
        let areas = aoc_utils::array::try_from_iter_exact(shapes.split("\n\n").map(|shape| {
            let Some((_, shape)) = shape.split_once(":\n") else {
                bail!("missing shape introducer line ending in colon");
            };
            let shape = aoc_utils::geometry::parse_ascii_map(shape)
                .with_context(|| anyhow!("failed to parse shape"))?;
            if shape.dim() != (3, 3) {
                bail!(
                    "expected shapes of up to 3x3, got {}x{}",
                    shape.dim().0,
                    shape.dim().1
                );
            }
            Ok(shape.iter().filter(|b| **b == b'#').count())
        }))?
        .map_err(|v| anyhow!("expected exactly 6 present shapes, got {}", v.len()))?;
        let problems = problems.lines().map(str::parse).try_collect()?;
        Ok(Door { areas, problems })
    }

    fn part1(&self) -> Result<usize> {
        let (conclusive, inconclusive): (Vec<bool>, HashSet<&Problem>) = self
            .problems
            .iter()
            .map(|problem| {
                rule_out_due_to_insufficient_area(problem, &self.areas)
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
        let presents = aoc_utils::array::try_from_iter_exact(shapes.split_whitespace().map(|s| {
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

fn rule_out_due_to_insufficient_area(problem: &Problem, areas: &[usize; 6]) -> Option<bool> {
    let required_area: usize = problem
        .presents
        .iter()
        .zip(areas)
        .map(|(count, area)| count * area)
        .sum();

    (required_area > problem.area()).then_some(false)
}

fn verify_with_trivial_packing(problem: &Problem) -> Option<bool> {
    let available_cells = (problem.dimensions[0] / 3) * (problem.dimensions[1] / 3);

    let total_presents: usize = problem.presents.iter().sum();

    (total_presents <= available_cells).then_some(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "\
0:
###
##.
##.

1:
###
##.
.##

2:
.##
###
##.

3:
##.
###
##.

4:
###
#..
###

5:
###
.#.
###

4x4: 0 0 0 0 2 0
12x5: 1 0 1 0 2 2
12x5: 1 0 1 0 3 2";

    const EXAMPLE_AREAS: [usize; 6] = [7, 7, 7, 7, 7, 7];
    const EXAMPLE_PROBLEMS: &[Problem] = &[
        Problem {
            dimensions: [4, 4],
            presents: [0, 0, 0, 0, 2, 0],
        },
        Problem {
            dimensions: [12, 5],
            presents: [1, 0, 1, 0, 2, 2],
        },
        Problem {
            dimensions: [12, 5],
            presents: [1, 0, 1, 0, 3, 2],
        },
    ];

    #[test]
    fn parse_example_input() {
        let Door { areas, problems } = Door::parse(EXAMPLE_INPUT).unwrap();
        assert_eq!(areas, EXAMPLE_AREAS);
        itertools::assert_equal(&problems, EXAMPLE_PROBLEMS);
    }

    #[test]
    fn none_of_the_example_problems_can_be_ruled_out_due_to_insufficient_area() {
        itertools::assert_equal(
            EXAMPLE_PROBLEMS
                .iter()
                .map(|problem| rule_out_due_to_insufficient_area(problem, &EXAMPLE_AREAS)),
            [None, None, None],
        );
    }

    #[test]
    fn none_of_the_example_problems_can_be_verified_with_trivial_packing() {
        itertools::assert_equal(
            EXAMPLE_PROBLEMS.iter().map(verify_with_trivial_packing),
            [None, None, None],
        );
    }
}
