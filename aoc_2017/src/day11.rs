use aoc_companion::prelude::*;
use aoc_utils::linalg::*;

use itertools::Itertools;
use thiserror::Error;

pub struct Door {
    steps: Vec<HexBasis>,
}

impl ParseInput<'_> for Door {
    type Error = ParseError;

    fn parse(input: &str) -> Result<Self, Self::Error> {
        parse_input(input).map(|steps| Self { steps })
    }
}

impl Part1 for Door {
    type Output = i32;
    type Error = std::convert::Infallible;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        let route = optimal_route(destination(&self.steps));
        Ok(route.norm_l1())
    }
}

impl Part2 for Door {
    type Output = i32;
    type Error = std::convert::Infallible;

    fn part2(&self) -> Result<Self::Output, Self::Error> {
        let furthest = furthest_point_along_path(&self.steps);
        Ok(furthest.norm_l1())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HexBasis {
    North,
    NorthWest,
    SouthWest,
    South,
    SouthEast,
    NorthEast,
}

impl HexBasis {
    fn to_coords(&self) -> Vector<i32, 3> {
        use HexBasis::*;
        match self {
            North => Vector([1, 0, 0]),
            NorthWest => Vector([0, 0, -1]),
            SouthWest => Vector([0, -1, 0]),
            South => Vector([-1, 0, 0]),
            SouthEast => Vector([0, 0, 1]),
            NorthEast => Vector([0, 1, 0]),
        }
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("The token '{token}' is not a valid hex direction")]
    NotAHexDirection { token: String },
}

fn parse_input(input: &str) -> Result<Vec<HexBasis>, ParseError> {
    use HexBasis::*;
    input
        .split(',')
        .map(|token| match token {
            "n" => Ok(North),
            "nw" => Ok(NorthWest),
            "sw" => Ok(SouthWest),
            "s" => Ok(South),
            "se" => Ok(SouthEast),
            "ne" => Ok(NorthEast),
            _ => Err(ParseError::NotAHexDirection {
                token: token.to_owned(),
            }),
        })
        .collect()
}

fn destination(steps: &[HexBasis]) -> Vector<i32, 3> {
    steps.iter().map(HexBasis::to_coords).sum()
}

fn optimal_route(destination: Vector<i32, 3>) -> Vector<i32, 3> {
    const NULL_SPACE: Vector<i32, 3> = Vector([1, -1, 1]);
    let (&lambda_min, &lambda_max) = (destination * NULL_SPACE)
        .iter()
        .minmax()
        .into_option()
        .unwrap();
    (lambda_min..=lambda_max)
        .map(|lambda| destination - NULL_SPACE * lambda)
        .min_by_key(Vector::<i32, 3>::norm_l1)
        .unwrap()
}

fn furthest_point_along_path(steps: &[HexBasis]) -> Vector<i32, 3> {
    steps
        .iter()
        .map(HexBasis::to_coords)
        .scan(Default::default(), |pos, v| {
            *pos += v;
            Some(*pos)
        })
        .map(optimal_route)
        .max_by_key(Vector::<i32, 3>::norm_l1)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use HexBasis::*;

    const EXAMPLE_STEPS_1: &[HexBasis] = &[NorthEast; 3];
    const EXAMPLE_STEPS_2: &[HexBasis] = &[NorthEast, NorthEast, SouthWest, SouthWest];
    const EXAMPLE_STEPS_3: &[HexBasis] = &[NorthEast, NorthEast, South, South];
    const EXAMPLE_STEPS_4: &[HexBasis] = &[SouthEast, SouthWest, SouthEast, SouthWest, SouthWest];

    const DESTINATION_1: Vector<i32, 3> = Vector([0, 3, 0]);
    const DESTINATION_2: Vector<i32, 3> = Vector([0, 0, 0]);
    const DESTINATION_3: Vector<i32, 3> = Vector([-2, 2, 0]);
    const DESTINATION_4: Vector<i32, 3> = Vector([0, -3, 2]);

    #[test]
    fn input_sequence_is_parsed() {
        assert_eq!(&parse_input("ne,ne,ne").unwrap(), EXAMPLE_STEPS_1);
        assert_eq!(&parse_input("ne,ne,sw,sw").unwrap(), EXAMPLE_STEPS_2);
        assert_eq!(&parse_input("ne,ne,s,s").unwrap(), EXAMPLE_STEPS_3);
        assert_eq!(&parse_input("se,sw,se,sw,sw").unwrap(), EXAMPLE_STEPS_4);
        assert_matches!(parse_input("se,w,nw"), Err(ParseError::NotAHexDirection { token }) if token == "w");
    }

    #[test]
    fn destination_is_found_by_following_steps() {
        assert_eq!(destination(EXAMPLE_STEPS_1), DESTINATION_1);
        assert_eq!(destination(EXAMPLE_STEPS_2), DESTINATION_2);
        assert_eq!(destination(EXAMPLE_STEPS_3), DESTINATION_3);
        assert_eq!(destination(EXAMPLE_STEPS_4), DESTINATION_4);
    }

    #[test]
    fn optimal_routes_are_found() {
        assert_eq!(optimal_route(DESTINATION_1), Vector([0, 3, 0]));
        assert_eq!(optimal_route(DESTINATION_2), Vector([0, 0, 0]));
        assert_eq!(optimal_route(DESTINATION_3), Vector([0, 0, 2]));
        assert_eq!(optimal_route(DESTINATION_4), Vector([-2, -1, 0]));
    }

    #[test]
    fn furthest_points_along_path_are_found() {
        assert_eq!(
            furthest_point_along_path(EXAMPLE_STEPS_1),
            Vector([0, 3, 0])
        );
        assert_eq!(
            furthest_point_along_path(EXAMPLE_STEPS_2),
            Vector([0, 2, 0])
        );
        assert_matches!(
            furthest_point_along_path(EXAMPLE_STEPS_3),
            Vector([0, 2, 0]) | Vector([0, 1, 1]) | Vector([0, 0, 2])
        );
        assert_eq!(
            furthest_point_along_path(EXAMPLE_STEPS_4),
            Vector([-2, -1, 0])
        );
    }
}
