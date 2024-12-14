use std::collections::HashSet;

use anyhow::bail;
use aoc_companion::prelude::*;
use aoc_utils::{geometry::Point, linalg::Vector};
use itertools::Itertools;

pub(crate) struct Door {
    robots: Vec<Robot>,
    bounds: Vector<i16, 2>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        Ok(Door {
            robots: input.lines().map(str::parse).try_collect()?,
            bounds: Vector([101, 103]),
        })
    }

    fn part1(&self) -> usize {
        let mut robots = self.robots.clone();
        step_n(&mut robots, 100, self.bounds);
        safety_factor(&robots, self.bounds)
    }

    fn part2(&self) -> usize {
        let critical_mass = (0.4 * self.robots.len() as f64) as usize;
        steps_to_large_cluster(&self.robots, self.bounds, critical_mass)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Robot {
    p: Vector<i16, 2>,
    v: Vector<i16, 2>,
}

impl std::str::FromStr for Robot {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let Some((pos_str, vel_str)) = s.trim().split_once(' ') else {
            bail!("Missing space separating position and velocity");
        };

        let Some(pos_str) = pos_str.strip_prefix("p=") else {
            bail!("Missing position introducer");
        };

        let Some(vel_str) = vel_str.strip_prefix("v=") else {
            bail!("Missing velocity introducer");
        };

        Ok(Robot {
            p: pos_str.parse()?,
            v: vel_str.parse()?,
        })
    }
}

impl Robot {
    fn step_n(&mut self, n: usize, bounds: Vector<i16, 2>) {
        self.p += self.v * n as i16;
        self.p += bounds * n as i16;
        self.p %= bounds;
    }

    fn quadrant(&self, bounds: Vector<i16, 2>) -> u8 {
        let half = bounds / 2;
        match self.p {
            Vector([x, y]) if x == half[0] || y == half[1] => 0,
            Vector([x, y]) if x < half[0] && y < half[1] => 1,
            Vector([x, _]) if x < half[0] => 2,
            Vector([_, y]) if y < half[1] => 3,
            _ => 4,
        }
    }
}

fn step_n(robots: &mut [Robot], n: usize, bounds: Vector<i16, 2>) {
    for robot in robots {
        robot.step_n(n, bounds);
    }
}

fn safety_factor(robots: &[Robot], bounds: Vector<i16, 2>) -> usize {
    let quadrant_counts = robots.iter().counts_by(|robot| robot.quadrant(bounds));
    quadrant_counts
        .into_iter()
        .filter_map(|(q, c)| (q != 0).then_some(c))
        .product()
}

fn largest_cluster(robots: &[Robot]) -> usize {
    let mut remaining: HashSet<_> = robots.iter().map(|robot| robot.p).collect();

    std::iter::from_fn(move || {
        let start = remaining.iter().next().cloned()?;
        remaining.remove(&start);
        let mut cluster = HashSet::new();
        let mut todo = vec![start];
        while let Some(p) = todo.pop() {
            cluster.insert(p);
            todo.extend(p.neighbors().filter(|q| remaining.remove(q)));
        }
        Some(cluster)
    })
    .map(|cluster| cluster.len())
    .max()
    .unwrap()
}

fn steps_to_large_cluster(robots: &[Robot], bounds: Vector<i16, 2>, critical_mass: usize) -> usize {
    let mut robots = robots.to_vec();
    std::iter::from_fn(move || {
        step_n(&mut robots, 1, bounds);
        Some(largest_cluster(&robots))
    })
    .position(|largest| largest >= critical_mass)
    .unwrap()
        + 1
}

#[cfg(test)]
mod tests {
    use itertools::assert_equal;

    use super::*;

    const EXAMPLE_INPUT: &str = "\
p=0,4 v=3,-3
p=6,3 v=-1,-3
p=10,3 v=-1,2
p=2,0 v=2,-1
p=0,0 v=1,3
p=3,0 v=-2,-2
p=7,6 v=-1,-3
p=3,0 v=-1,-2
p=9,3 v=2,3
p=7,3 v=-1,2
p=2,4 v=2,-3
p=9,5 v=-3,-3";

    const EXAMPLE_ROBOTS: &[Robot] = &[
        Robot {
            p: Vector([0, 4]),
            v: Vector([3, -3]),
        },
        Robot {
            p: Vector([6, 3]),
            v: Vector([-1, -3]),
        },
        Robot {
            p: Vector([10, 3]),
            v: Vector([-1, 2]),
        },
        Robot {
            p: Vector([2, 0]),
            v: Vector([2, -1]),
        },
        Robot {
            p: Vector([0, 0]),
            v: Vector([1, 3]),
        },
        Robot {
            p: Vector([3, 0]),
            v: Vector([-2, -2]),
        },
        Robot {
            p: Vector([7, 6]),
            v: Vector([-1, -3]),
        },
        Robot {
            p: Vector([3, 0]),
            v: Vector([-1, -2]),
        },
        Robot {
            p: Vector([9, 3]),
            v: Vector([2, 3]),
        },
        Robot {
            p: Vector([7, 3]),
            v: Vector([-1, 2]),
        },
        Robot {
            p: Vector([2, 4]),
            v: Vector([2, -3]),
        },
        Robot {
            p: Vector([9, 5]),
            v: Vector([-3, -3]),
        },
    ];
    const EXAMPLE_BOUNDS: Vector<i16, 2> = Vector([11, 7]);

    #[test]
    fn parse_example_robots() {
        assert_equal(
            EXAMPLE_INPUT
                .lines()
                .map(|line| line.parse::<Robot>().unwrap()),
            EXAMPLE_ROBOTS.iter().copied(),
        );
    }

    #[test]
    fn safety_factor_after_100_steps() {
        let mut robots = EXAMPLE_ROBOTS.to_vec();
        step_n(&mut robots, 100, EXAMPLE_BOUNDS);
        assert_eq!(safety_factor(&robots, EXAMPLE_BOUNDS), 12);
    }
}
