use std::collections::HashMap;

use aoc_companion::prelude::*;
use aoc_utils::array;
use itertools::{Either, Itertools};

type Pattern2 = [bool; 4];
type Pattern3 = [bool; 9];
type Pattern4 = [bool; 16];

pub(crate) struct Door {
    rules2: HashMap<Pattern2, Pattern3>,
    rules3: HashMap<Pattern3, Pattern4>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Self {
        let (rules2, rules3) = input.lines().partition_map(|line| {
            if &line[6..8] == "=>" {
                Either::Left((
                    parse_pattern::<4>(&line[0..5]),
                    parse_pattern::<9>(&line[9..]),
                ))
            } else if &line[12..14] == "=>" {
                Either::Right((
                    parse_pattern::<9>(&line[0..11]),
                    parse_pattern::<16>(&line[15..]),
                ))
            } else {
                unreachable!()
            }
        });

        Self { rules2, rules3 }
    }

    fn part1(&self) -> usize {
        self.lit_pixels(5)
    }

    fn part2(&self) -> usize {
        self.lit_pixels(18)
    }
}

fn parse_pattern<const N: usize>(s: &str) -> [bool; N] {
    array::from_iter_exact(s.chars().filter(|&c| c != '/').map(|c| match c {
        '.' => false,
        '#' => true,
        _ => unreachable!(),
    }))
    .unwrap()
}

const INIT_PIC: Pattern3 = [false, true, false, false, false, true, true, true, true];

impl Door {
    fn lit_pixels(&self, iterations: usize) -> usize {
        self.lit_pixels_recursive(
            ndarray::ArrayView::from_shape((3, 3), &INIT_PIC).unwrap(),
            iterations,
        )
    }

    fn lit_pixels_recursive(&self, pic: ndarray::ArrayView2<bool>, iterations: usize) -> usize {
        let dim = pic.shape()[0];
        if iterations == 0 {
            pic.into_iter().filter(|&&x| x).count()
        } else if dim % 2 == 0 {
            let mut new_pic = ndarray::Array2::from_elem([dim / 2 * 3, dim / 2 * 3], false);
            pic.exact_chunks((2, 2))
                .into_iter()
                .zip(new_pic.exact_chunks_mut((3, 3)))
                .for_each(|(chunk_2x2, mut chunk_3x3)| {
                    let in_pattern: Pattern2 =
                        array::from_iter_exact(chunk_2x2.iter().copied()).unwrap();
                    let out_pattern = dihedral_2x2(in_pattern)
                        .iter()
                        .find_map(|img| self.rules2.get(img).cloned())
                        .unwrap();
                    chunk_3x3
                        .assign(&ndarray::ArrayView2::from_shape((3, 3), &out_pattern).unwrap());
                });
            self.lit_pixels_recursive(new_pic.view(), iterations - 1)
        } else {
            let mut new_pic = ndarray::Array2::from_elem([dim / 3 * 4, dim / 3 * 4], false);
            pic.exact_chunks((3, 3))
                .into_iter()
                .zip(new_pic.exact_chunks_mut((4, 4)))
                .for_each(|(chunk_3x3, mut chunk_4x4)| {
                    let in_pattern: Pattern3 =
                        array::from_iter_exact(chunk_3x3.iter().copied()).unwrap();
                    let out_pattern = dihedral_3x3(in_pattern)
                        .iter()
                        .find_map(|img| self.rules3.get(img).cloned())
                        .unwrap();
                    chunk_4x4
                        .assign(&ndarray::ArrayView2::from_shape((4, 4), &out_pattern).unwrap());
                });
            self.lit_pixels_recursive(new_pic.view(), iterations - 1)
        }
    }
}

fn dihedral_2x2([a, b, c, d]: Pattern2) -> [Pattern2; 8] {
    [
        [a, b, c, d],
        [b, d, a, c],
        [d, c, b, a],
        [c, a, d, b],
        [c, d, a, b],
        [d, b, c, a],
        [b, a, c, d],
        [a, c, b, d],
    ]
}

fn dihedral_3x3([a, b, c, d, e, f, g, h, i]: Pattern3) -> [Pattern3; 8] {
    [
        [a, b, c, d, e, f, g, h, i],
        [c, f, i, b, e, h, a, d, g],
        [i, h, g, f, e, d, c, b, a],
        [g, d, a, h, e, b, i, f, c],
        [g, h, i, d, e, f, a, b, c],
        [i, f, c, h, e, b, g, d, a],
        [c, b, a, f, e, d, i, h, g],
        [a, d, g, b, e, h, c, f, i],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "../.# => ##./#../...
.#./..#/### => #..#/..../..../#..#";

    #[test]
    fn parse_example_input() {
        let Door { rules2, rules3 } = Door::parse(EXAMPLE_INPUT);
        assert_eq!(
            rules2,
            HashMap::from([(
                [false, false, false, true],
                [true, true, false, true, false, false, false, false, false]
            )])
        );
        assert_eq!(
            rules3,
            HashMap::from([(
                [false, true, false, false, false, true, true, true, true],
                [
                    true, false, false, true, false, false, false, false, false, false, false,
                    false, true, false, false, true
                ],
            )])
        );
    }

    #[test]
    fn lit_pixels_after_two_iterations() {
        assert_eq!(Door::parse(EXAMPLE_INPUT).lit_pixels(2), 12);
    }
}
