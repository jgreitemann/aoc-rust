use anyhow::Context as _;
use aoc_companion::prelude::*;
use itertools::Itertools as _;
use num_traits::Euclid as _;

pub(crate) struct Door {
    rotations: Vec<i32>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        Ok(Door {
            rotations: input
                .lines()
                .map(|line| {
                    let Some((dir, angle)) = line.split_at_checked(1) else {
                        anyhow::bail!("unexpected empty line");
                    };
                    let angle: i32 = angle
                        .parse()
                        .with_context(|| anyhow::anyhow!("invalid rotation angle"))?;
                    match dir {
                        "L" => Ok(-angle),
                        "R" => Ok(angle),
                        _ => Err(anyhow::anyhow!("invalid rotation direction {dir:?}")),
                    }
                })
                .try_collect()?,
        })
    }

    fn part1(&self) -> usize {
        intermediate_angles(self.rotations.iter().copied())
            .filter(|angle| *angle == 0)
            .count()
    }

    fn part2(&self) -> i32 {
        dial_crossings(self.rotations.iter().copied()).sum()
    }
}

fn intermediate_angles(rotations: impl IntoIterator<Item = i32>) -> impl Iterator<Item = i32> {
    rotations.into_iter().scan(50, |angle, rot| {
        *angle = (*angle + rot).rem_euclid(100);
        Some(*angle)
    })
}

fn dial_crossings(rotations: impl IntoIterator<Item = i32>) -> impl Iterator<Item = i32> {
    rotations.into_iter().scan(50, |angle, rot| {
        let (div, new_angle) = (*angle + rot).div_rem_euclid(&100);
        let correction = if rot < 0 {
            (new_angle == 0) as i32 - (*angle == 0) as i32
        } else {
            0
        };
        *angle = new_angle;
        Some(div.abs() + correction)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_ROTATIONS: &[i32] = &[-68, -30, 48, -5, 60, -55, -1, -99, 14, -82];

    #[test]
    fn intermediate_angles_in_example() {
        itertools::assert_equal(
            intermediate_angles(EXAMPLE_ROTATIONS.iter().copied()),
            [82, 52, 0, 95, 55, 0, 99, 0, 14, 32],
        );
    }

    #[test]
    fn times_dial_reaches_zero() {
        itertools::assert_equal(
            dial_crossings(EXAMPLE_ROTATIONS.iter().copied()),
            [1, 0, 1, 0, 1, 1, 0, 1, 0, 1],
        );
    }
}
