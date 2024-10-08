use std::num::ParseIntError;

use aoc_companion::prelude::*;

use thiserror::Error;

pub struct Door(Vec<Layer>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Layer {
    depth: usize,
    range: usize,
}

impl Door {
    fn total_severity(&self) -> usize {
        self.0.iter().map(Layer::severity).sum()
    }
}

impl Layer {
    fn will_catch(&self, delay: usize) -> bool {
        (self.depth + delay) % (2 * (self.range - 1)) == 0
    }

    fn severity(&self) -> usize {
        if self.will_catch(0) {
            self.depth * self.range
        } else {
            0
        }
    }
}

fn min_safe_delay(layers: &[Layer]) -> usize {
    (0..)
        .find(|&delay| layers.iter().all(|layer| !layer.will_catch(delay)))
        .unwrap()
}

impl ParseInput<'_> for Door {
    type Error = ParseError;

    fn parse(input: &str) -> Result<Self, Self::Error> {
        parse_input(input).map(Door)
    }
}

impl Part1 for Door {
    type Output = usize;
    type Error = std::convert::Infallible;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        Ok(self.total_severity())
    }
}

impl Part2 for Door {
    type Output = usize;
    type Error = std::convert::Infallible;

    fn part2(&self) -> Result<Self::Output, Self::Error> {
        Ok(min_safe_delay(&self.0))
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("A line in the input does not contain a colon")]
    MissingColon,
    #[error("Depth or range are not numeric")]
    InvalidInteger(#[from] ParseIntError),
}

fn parse_input(input: &str) -> Result<Vec<Layer>, ParseError> {
    input
        .lines()
        .map(|line| {
            line.split_once(':')
                .ok_or(ParseError::MissingColon)
                .and_then(|(depth, range)| {
                    Ok(Layer {
                        depth: depth.trim().parse()?,
                        range: range.trim().parse()?,
                    })
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = r"0: 3
1: 2
4: 4
6: 4
";

    const EXAMPLE_LAYERS: [Layer; 4] = [
        Layer { depth: 0, range: 3 },
        Layer { depth: 1, range: 2 },
        Layer { depth: 4, range: 4 },
        Layer { depth: 6, range: 4 },
    ];

    #[test]
    fn example_input_parsed() {
        assert_eq!(parse_input(EXAMPLE_INPUT).unwrap(), EXAMPLE_LAYERS);
    }

    #[test]
    fn example_layer_severities() {
        itertools::assert_equal(EXAMPLE_LAYERS.iter().map(Layer::severity), [0, 0, 0, 24]);
    }

    #[test]
    fn example_min_safe_delay() {
        assert_eq!(min_safe_delay(&EXAMPLE_LAYERS), 10);
    }
}
