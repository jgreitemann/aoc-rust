use std::{fmt::Write, iter::repeat_n};

use anyhow::bail;
use aoc_companion::prelude::*;
use aoc_utils::{cache::cached, linalg::Vector};
use itertools::Itertools;

pub(crate) struct Door {
    codes: Vec<Vec<NumericButton>>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        use NumericButton::*;
        let codes = input
            .lines()
            .map(|line| {
                line.as_bytes()
                    .iter()
                    .map(|c| {
                        Ok(match c {
                            b'0' => N0,
                            b'1' => N1,
                            b'2' => N2,
                            b'3' => N3,
                            b'4' => N4,
                            b'5' => N5,
                            b'6' => N6,
                            b'7' => N7,
                            b'8' => N8,
                            b'9' => N9,
                            b'A' => A,
                            _ => bail!("unknown numeric key"),
                        })
                    })
                    .try_collect()
            })
            .try_collect()?;
        Ok(Door { codes })
    }

    fn part1(&self) -> usize {
        self.codes.iter().map(|code| complexity::<2>(code)).sum()
    }

    fn part2(&self) -> usize {
        self.codes.iter().map(|code| complexity::<25>(code)).sum()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NumericButton {
    N0,
    N1,
    N2,
    N3,
    N4,
    N5,
    N6,
    N7,
    N8,
    N9,
    A,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DirectionalButton {
    Up,
    A,
    Left,
    Down,
    Right,
}

impl NumericButton {
    fn coords(&self) -> Vector<i8, 2> {
        match self {
            NumericButton::N0 => Vector([1, 0]),
            NumericButton::N1 => Vector([0, 1]),
            NumericButton::N2 => Vector([1, 1]),
            NumericButton::N3 => Vector([2, 1]),
            NumericButton::N4 => Vector([0, 2]),
            NumericButton::N5 => Vector([1, 2]),
            NumericButton::N6 => Vector([2, 2]),
            NumericButton::N7 => Vector([0, 3]),
            NumericButton::N8 => Vector([1, 3]),
            NumericButton::N9 => Vector([2, 3]),
            NumericButton::A => Vector([2, 0]),
        }
    }
}

impl std::fmt::Display for NumericButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(match self {
            NumericButton::N0 => '0',
            NumericButton::N1 => '1',
            NumericButton::N2 => '2',
            NumericButton::N3 => '3',
            NumericButton::N4 => '4',
            NumericButton::N5 => '5',
            NumericButton::N6 => '6',
            NumericButton::N7 => '7',
            NumericButton::N8 => '8',
            NumericButton::N9 => '9',
            NumericButton::A => 'A',
        })
    }
}

impl std::fmt::Display for DirectionalButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(match self {
            DirectionalButton::Left => '<',
            DirectionalButton::Down => 'v',
            DirectionalButton::Right => '>',
            DirectionalButton::Up => '^',
            DirectionalButton::A => 'A',
        })
    }
}

fn shortest_moves(
    from: Vector<i8, 2>,
    to: Vector<i8, 2>,
) -> impl IntoIterator<
    IntoIter = impl Iterator<Item = impl Iterator<Item = DirectionalButton> + Clone> + Clone,
    Item = impl Iterator<Item = DirectionalButton> + Clone,
> + Clone {
    let offset = to - from;
    [
        (from[1] != 0 || to[0] != 0).then_some(
            Iterator::chain(
                repeat_n(
                    if offset[0] < 0 {
                        DirectionalButton::Left
                    } else {
                        DirectionalButton::Right
                    },
                    offset[0].unsigned_abs() as usize,
                ),
                repeat_n(
                    if offset[1] < 0 {
                        DirectionalButton::Down
                    } else {
                        DirectionalButton::Up
                    },
                    offset[1].unsigned_abs() as usize,
                ),
            )
            .chain(std::iter::once(DirectionalButton::A)),
        ),
        ((from[0] != 0 || to[1] != 0) && from[0] != to[0] && from[1] != to[1]).then_some(
            Iterator::chain(
                repeat_n(
                    if offset[1] < 0 {
                        DirectionalButton::Down
                    } else {
                        DirectionalButton::Up
                    },
                    offset[1].unsigned_abs() as usize,
                ),
                repeat_n(
                    if offset[0] < 0 {
                        DirectionalButton::Left
                    } else {
                        DirectionalButton::Right
                    },
                    offset[0].unsigned_abs() as usize,
                ),
            )
            .chain(std::iter::once(DirectionalButton::A)),
        ),
    ]
    .into_iter()
    .flatten()
}

fn numeric_keypad_moves(
    desired_code: &[NumericButton],
) -> impl Iterator<Item = impl IntoIterator<Item = DirectionalButton> + Clone> {
    std::iter::once(NumericButton::A)
        .chain(desired_code.iter().copied())
        .tuple_windows()
        .map(|(from, to)| shortest_moves(from.coords(), to.coords()))
        .multi_cartesian_product()
        .map(|s| s.into_iter().flatten().collect_vec())
}

fn shortest_directional_moves(
    from: DirectionalButton,
    to: DirectionalButton,
) -> &'static [DirectionalButton] {
    use DirectionalButton::*;
    match (from, to) {
        (Up, Up) => &[A],
        (Up, A) => &[Right, A],
        (Up, Left) => &[Down, Left, A],
        (Up, Down) => &[Down, A],
        (Up, Right) => &[Down, Right, A],
        (A, Up) => &[Left, A],
        (A, A) => &[A],
        (A, Left) => &[Down, Left, Left, A],
        (A, Down) => &[Left, Down, A],
        (A, Right) => &[Down, A],
        (Left, Up) => &[Right, Up, A],
        (Left, A) => &[Right, Right, Up, A],
        (Left, Left) => &[A],
        (Left, Down) => &[Right, A],
        (Left, Right) => &[Right, Right, A],
        (Down, Up) => &[Up, A],
        (Down, A) => &[Up, Right, A],
        (Down, Left) => &[Left, A],
        (Down, Down) => &[A],
        (Down, Right) => &[Right, A],
        (Right, Up) => &[Left, Up, A],
        (Right, A) => &[Up, A],
        (Right, Left) => &[Left, Left, A],
        (Right, Down) => &[Left, A],
        (Right, Right) => &[A],
    }
}

fn directional_keypad_moves(desired_buttons: Vec<DirectionalButton>) -> Vec<DirectionalButton> {
    std::iter::once(DirectionalButton::A)
        .chain(desired_buttons)
        .tuple_windows()
        .flat_map(|(from, to)| shortest_directional_moves(from, to).iter().copied())
        .collect()
}

fn shortest_seq<const N: usize>(desired_code: &[NumericButton]) -> usize {
    let mut cached_subseq_len = cached(|subseq, _| subseq_len(subseq, N / 2));
    numeric_keypad_moves(desired_code)
        .map(|moves| -> usize {
            let moves = moves.into_iter().collect_vec();
            (0..(N - N / 2))
                .fold(moves, |v, _| directional_keypad_moves(v))
                .split_inclusive(|&e| e == DirectionalButton::A)
                .map(|subseq| cached_subseq_len(subseq.to_vec()))
                .sum()
        })
        .min()
        .unwrap()
}

fn subseq_len(subseq: Vec<DirectionalButton>, m: usize) -> usize {
    (0..m)
        .fold(subseq, |v, _| directional_keypad_moves(v))
        .len()
}

fn complexity<const N: usize>(desired_code: &[NumericButton]) -> usize {
    desired_code
        .iter()
        .join("")
        .strip_suffix("A")
        .unwrap()
        .parse::<usize>()
        .unwrap()
        * shortest_seq::<N>(desired_code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directional_moves_for_numeric_code() {
        use NumericButton::*;
        assert_eq!(
            numeric_keypad_moves(&[N0, N2, N9, A])
                .map(|i| i.into_iter().map(|b| b.to_string()).join(""))
                .collect_vec(),
            ["<A^A>^^AvvvA", "<A^A^^>AvvvA"]
        );
    }

    #[test]
    fn directional_moves_for_directional_code() {
        use DirectionalButton::*;
        assert_eq!(
            directional_keypad_moves(vec![Left, A, Up, A, Right, Up, Up, A, Down, Down, Down, A])
                .into_iter()
                .map(|b| b.to_string())
                .join(""),
            "v<<A>>^A<A>AvA<^AA>A<vAAA^>A"
        );
    }

    #[test]
    fn shortest_manuel_keypad_seq_len() {
        use NumericButton::*;
        assert_eq!(shortest_seq::<2>(&[N0, N2, N9, A]), 68);
        assert_eq!(shortest_seq::<2>(&[N9, N8, N0, A]), 60);
        assert_eq!(shortest_seq::<2>(&[N1, N7, N9, A]), 68);
        assert_eq!(shortest_seq::<2>(&[N4, N5, N6, A]), 64);
        assert_eq!(shortest_seq::<2>(&[N3, N7, N9, A]), 64);
    }

    #[test]
    fn complexity_of_example_codes() {
        use NumericButton::*;
        assert_eq!(complexity::<2>(&[N0, N2, N9, A]), 68 * 29);
        assert_eq!(complexity::<2>(&[N9, N8, N0, A]), 60 * 980);
        assert_eq!(complexity::<2>(&[N1, N7, N9, A]), 68 * 179);
        assert_eq!(complexity::<2>(&[N4, N5, N6, A]), 64 * 456);
        assert_eq!(complexity::<2>(&[N3, N7, N9, A]), 64 * 379);
    }
}
