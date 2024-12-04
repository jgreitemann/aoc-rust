use aoc_companion::prelude::*;
use aoc_utils::{
    array::{try_from_fn, try_map},
    geometry::Point,
    linalg::Vector,
};

pub(crate) struct Door {
    array: ndarray::Array2<u8>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Self {
        Self {
            array: parse_input(input),
        }
    }

    fn part1(&self) -> usize {
        count_xmas(self.array.view())
    }

    fn part2(&self) -> impl door::IntoResult {
        count_cross_of_mas(self.array.view())
    }
}

fn parse_input(input: &str) -> ndarray::Array2<u8> {
    let rows = input.lines().count();
    let cols = input.lines().next().unwrap().len();
    ndarray::Array2::from_shape_vec(
        [rows, cols],
        input.bytes().filter(|b| !b.is_ascii_whitespace()).collect(),
    )
    .unwrap()
}

fn count_xmas(view: ndarray::ArrayView2<u8>) -> usize {
    view.indexed_iter()
        .filter(|(_, b)| **b == b'X')
        .flat_map(|(first, _)| {
            let first = Vector(first.into());
            first
                .neighbors()
                .filter_map(move |second| extrapolate::<4>(first, second))
        })
        .filter_map(|line| try_map(line, |idx| view.get(idx).copied().ok_or(())).ok())
        .filter(|elems| elems == b"XMAS")
        .count()
}

fn count_cross_of_mas(view: ndarray::ArrayView2<u8>) -> usize {
    view.indexed_iter()
        .filter(|(_, b)| **b == b'A')
        .filter(|&(center, _)| {
            let center = Vector(center.into());
            center
                .next_nearest_neighbors()
                .filter_map(move |nnn| extrapolate::<3>(nnn, center))
                .filter_map(|line| try_map(line, |idx| view.get(idx).copied().ok_or(())).ok())
                .filter(|elems| elems == b"MAS")
                .count()
                == 2
        })
        .count()
}

fn extrapolate<const N: usize>(
    first: Vector<usize, 2>,
    second: Vector<usize, 2>,
) -> Option<[Vector<usize, 2>; N]> {
    let first: Vector<isize, 2> = Vector(try_map(first.0, isize::try_from).unwrap());
    let second: Vector<isize, 2> = Vector(try_map(second.0, isize::try_from).unwrap());
    let delta = second - first;

    try_from_fn(|idx| try_map((first + delta * idx as isize).0, |x| x.try_into()).map(Vector)).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "\
MMMSXXMASM
MSAMXMSMSA
AMXSXMAAMM
MSAMASMSMX
XMASAMXAMM
XXAMMXXAMA
SMSMSASXSS
SAXAMASAAA
MAMMMXMMMM
MXMXAXMASX";

    fn example_array() -> ndarray::Array2<u8> {
        ndarray::array![
            [b'M', b'M', b'M', b'S', b'X', b'X', b'M', b'A', b'S', b'M',],
            [b'M', b'S', b'A', b'M', b'X', b'M', b'S', b'M', b'S', b'A',],
            [b'A', b'M', b'X', b'S', b'X', b'M', b'A', b'A', b'M', b'M',],
            [b'M', b'S', b'A', b'M', b'A', b'S', b'M', b'S', b'M', b'X',],
            [b'X', b'M', b'A', b'S', b'A', b'M', b'X', b'A', b'M', b'M',],
            [b'X', b'X', b'A', b'M', b'M', b'X', b'X', b'A', b'M', b'A',],
            [b'S', b'M', b'S', b'M', b'S', b'A', b'S', b'X', b'S', b'S',],
            [b'S', b'A', b'X', b'A', b'M', b'A', b'S', b'A', b'A', b'A',],
            [b'M', b'A', b'M', b'M', b'M', b'X', b'M', b'M', b'M', b'M',],
            [b'M', b'X', b'M', b'X', b'A', b'X', b'M', b'A', b'S', b'X',],
        ]
    }

    #[test]
    fn parse_input_as_array() {
        assert_eq!(parse_input(EXAMPLE_INPUT), example_array());
    }

    #[test]
    fn example_xmas_count() {
        assert_eq!(count_xmas(example_array().view()), 18);
    }

    #[test]
    fn example_cross_of_mas_count() {
        assert_eq!(count_cross_of_mas(example_array().view()), 9);
    }
}
