use aoc_companion::prelude::*;
use aoc_utils::geometry::Point;
use aoc_utils::linalg::{ParseVectorError, Vector};

use itertools::Itertools;

use std::collections::HashSet;
use std::num::ParseIntError;

type Voxel = Vector<i8, 3>;

pub(crate) struct Door {
    voxels: HashSet<Voxel>,
}

impl<'input> ParseInput<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseVectorError<ParseIntError>> {
        parse_input(input).map(|voxels| Door { voxels })
    }
}

impl Part1 for Door {
    fn part1(&self) -> usize {
        total_surface_area(&self.voxels)
    }
}

impl Part2 for Door {
    fn part2(&self) -> usize {
        exterior_surface_area(&self.voxels)
    }
}

fn parse_input(input: &str) -> Result<HashSet<Voxel>, ParseVectorError<ParseIntError>> {
    input.lines().map(str::parse).try_collect()
}

fn total_surface_area(voxels: &HashSet<Voxel>) -> usize {
    surface_area(voxels, |p| !voxels.contains(p))
}

fn exterior_surface_area(voxels: &HashSet<Voxel>) -> usize {
    let exterior = exterior(voxels);
    surface_area(voxels, move |p| exterior.contains(p))
}

fn surface_area(voxels: &HashSet<Voxel>, is_outside: impl Fn(&Voxel) -> bool) -> usize {
    voxels
        .iter()
        .map(|v| v.nearest_neighbors().filter(&is_outside).count())
        .sum()
}

fn exterior(voxels: &HashSet<Voxel>) -> HashSet<Voxel> {
    let x_range = -1..=(voxels.iter().map(|v| v[0]).max().unwrap() + 1);
    let y_range = -1..=(voxels.iter().map(|v| v[1]).max().unwrap() + 1);
    let z_range = -1..=(voxels.iter().map(|v| v[2]).max().unwrap() + 1);

    let mut exterior = HashSet::new();
    let mut stack = vec![Voxel::new()];
    while let Some(p) = stack.pop() {
        if x_range.contains(&p[0])
            && y_range.contains(&p[1])
            && z_range.contains(&p[2])
            && !voxels.contains(&p)
            && !exterior.contains(&p)
        {
            exterior.insert(p);
            stack.extend(p.nearest_neighbors());
        }
    }

    exterior
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_can_be_parsed() {
        assert_eq!(
            parse_input(EXAMPLE_INPUT).unwrap(),
            HashSet::from(EXAMPLE_VOXELS)
        );
    }

    #[test]
    fn example_total_surface_area() {
        assert_eq!(total_surface_area(&HashSet::from(EXAMPLE_VOXELS)), 64);
    }

    #[test]
    fn example_exterior_surface_area() {
        assert_eq!(exterior_surface_area(&HashSet::from(EXAMPLE_VOXELS)), 58);
    }

    const EXAMPLE_INPUT: &str = "\
        2,2,2
        1,2,2
        3,2,2
        2,1,2
        2,3,2
        2,2,1
        2,2,3
        2,2,4
        2,2,6
        1,2,5
        3,2,5
        2,1,5
        2,3,5";

    const EXAMPLE_VOXELS: [Voxel; 13] = [
        Vector([2, 2, 2]),
        Vector([1, 2, 2]),
        Vector([3, 2, 2]),
        Vector([2, 1, 2]),
        Vector([2, 3, 2]),
        Vector([2, 2, 1]),
        Vector([2, 2, 3]),
        Vector([2, 2, 4]),
        Vector([2, 2, 6]),
        Vector([1, 2, 5]),
        Vector([3, 2, 5]),
        Vector([2, 1, 5]),
        Vector([2, 3, 5]),
    ];
}
