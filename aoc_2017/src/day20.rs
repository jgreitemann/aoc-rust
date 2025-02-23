use anyhow::{bail, Context};
use aoc_companion::prelude::*;
use aoc_utils::linalg::Vector;
use itertools::{iterate, Itertools};

pub(crate) struct Door {
    particles: Vec<Particle>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        Ok(Self {
            particles: input.lines().map(str::parse).try_collect()?,
        })
    }

    fn part1(&self) -> usize {
        closest_particle_long_term(&self.particles)
    }

    fn part2(&self) -> usize {
        evolve_and_prune(&self.particles, 10_000)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Particle {
    p: Vector<i32, 3>,
    v: Vector<i32, 3>,
    a: Vector<i32, 3>,
}

impl std::str::FromStr for Particle {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let Some((pos_str, rest_str)) = s.split_once(">, ") else {
            bail!("Missing comma separating position from velocity");
        };
        let Some((vel_str, acc_str)) = rest_str.split_once(">, ") else {
            bail!("Missing comma separating velocity from acceleration");
        };
        let Some(pos_str) = pos_str.strip_prefix("p=<") else {
            bail!("Missing position introducer");
        };
        let Some(vel_str) = vel_str.strip_prefix("v=<") else {
            bail!("Missing velocity introducer");
        };
        let Some(acc_str) = acc_str.strip_prefix("a=<") else {
            bail!("Missing acceleration introducer");
        };
        let Some(acc_str) = acc_str.strip_suffix(">") else {
            bail!("Missing closing angle bracket at the end");
        };
        Ok(Particle {
            p: pos_str.trim().parse().context("Cannot parse position")?,
            v: vel_str.trim().parse().context("Cannot parse velocity")?,
            a: acc_str
                .trim()
                .parse()
                .context("Cannot parse acceleration")?,
        })
    }
}

fn closest_particle_long_term(particles: &[Particle]) -> usize {
    particles
        .iter()
        .enumerate()
        .min_set_by_key(|(_, Particle { a, .. })| a.norm_l2_sq())
        .into_iter()
        .min_set_by_key(|&(_, &Particle { v, a, .. })| v.dot(a))
        .into_iter()
        .min_set_by_key(|&(_, &Particle { p, v, a })| p.dot(a) + v.norm_l2_sq())
        .into_iter()
        .min_set_by_key(|&(_, &Particle { p, v, .. })| p.dot(v))
        .into_iter()
        .min_by_key(|&(_, &Particle { p, .. })| p.norm_l2_sq())
        .unwrap()
        .0
}

fn particle_positions(particle: &Particle) -> impl Iterator<Item = Vector<i32, 3>> {
    let Particle { p, v, a } = particle.clone();
    iterate((p, v), move |&(p, v)| {
        let v = v + a;
        let p = p + v;
        (p, v)
    })
    .map(|(p, _)| p)
}

fn evolve_and_prune(particles: &[Particle], time: usize) -> usize {
    let mut p_iters = particles.iter().map(particle_positions).collect_vec();
    for _ in 0..time {
        let colliders = (0..p_iters.len()).into_group_map_by(|idx| p_iters[*idx].next().unwrap());
        let mut to_remove = colliders
            .into_values()
            .filter(|colliders| colliders.len() > 1)
            .flatten()
            .collect_vec();
        to_remove.sort();
        to_remove.reverse();
        for idx in to_remove {
            let _ = p_iters.remove(idx);
        }
    }

    p_iters.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    const EXAMPLE_INPUT: &str = "\
p=< 3,0,0>, v=< 2,0,0>, a=<-1,0,0>
p=< 4,0,0>, v=< 0,0,0>, a=<-2,0,0>";

    const EXAMPLE_PARTICLES: &[Particle] = &[
        Particle {
            p: Vector([3, 0, 0]),
            v: Vector([2, 0, 0]),
            a: Vector([-1, 0, 0]),
        },
        Particle {
            p: Vector([4, 0, 0]),
            v: Vector([0, 0, 0]),
            a: Vector([-2, 0, 0]),
        },
    ];

    const EXAMPLE_PARTICLES_2: &[Particle] = &[
        Particle {
            p: Vector([-6, 0, 0]),
            v: Vector([3, 0, 0]),
            a: Vector([0, 0, 0]),
        },
        Particle {
            p: Vector([-4, 0, 0]),
            v: Vector([2, 0, 0]),
            a: Vector([0, 0, 0]),
        },
        Particle {
            p: Vector([-2, 0, 0]),
            v: Vector([1, 0, 0]),
            a: Vector([0, 0, 0]),
        },
        Particle {
            p: Vector([3, 0, 0]),
            v: Vector([-1, 0, 0]),
            a: Vector([0, 0, 0]),
        },
    ];

    #[test]
    fn parse_example_input() {
        assert_eq!(
            EXAMPLE_INPUT
                .lines()
                .map(Particle::from_str)
                .collect::<Result<Vec<_>, _>>()
                .unwrap(),
            EXAMPLE_PARTICLES
        );
    }

    #[test]
    fn particles_with_different_abs_acceleration() {
        assert_eq!(closest_particle_long_term(EXAMPLE_PARTICLES), 0);
    }

    #[test]
    fn particles_with_same_abs_acceleration() {
        assert_eq!(
            closest_particle_long_term(&[
                Particle {
                    p: Vector([3, 0, 0]),
                    v: Vector([2, 0, 0]),
                    a: Vector([2, 0, 0]),
                },
                Particle {
                    p: Vector([4, 0, 0]),
                    v: Vector([0, 0, 0]),
                    a: Vector([-2, 0, 0]),
                },
            ]),
            1
        );
    }

    #[test]
    fn number_of_particles_left_after_collisions() {
        assert_eq!(evolve_and_prune(EXAMPLE_PARTICLES_2, 2), 4);
        assert_eq!(evolve_and_prune(EXAMPLE_PARTICLES_2, 3), 1);
    }
}
