use std::num::ParseIntError;

use aoc_companion::prelude::*;
use aoc_utils::linalg::{ParseVectorError, Vector};
use itertools::Itertools as _;

pub(crate) struct Door {
    boxes: Vec<Vector<i64, 3>>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseVectorError<ParseIntError>> {
        input
            .lines()
            .map(str::parse)
            .try_collect()
            .map(|boxes| Door { boxes })
    }

    fn part1(&self) -> usize {
        networks(&self.boxes, 1000)
            .iter()
            .counts()
            .into_values()
            .sorted()
            .rev()
            .take(3)
            .product()
    }

    fn part2(&self) -> i64 {
        let (Vector([x1, _, _]), Vector([x2, _, _])) = final_connection(&self.boxes);
        x1 * x2
    }
}

fn networks(boxes: &[Vector<i64, 3>], n_connect: usize) -> Vec<usize> {
    boxes
        .iter()
        .enumerate()
        .tuple_combinations()
        .sorted_unstable_by_key(|((_, pi), (_, pj))| (**pi - **pj).norm_l2_sq())
        .take(n_connect)
        .fold(
            (0..boxes.len()).collect_vec(),
            |mut assoc, ((i, _), (j, _))| {
                update_network_associations(&mut assoc, i, j);
                assoc
            },
        )
}

fn final_connection(boxes: &[Vector<i64, 3>]) -> (Vector<i64, 3>, Vector<i64, 3>) {
    boxes
        .iter()
        .enumerate()
        .tuple_combinations()
        .sorted_unstable_by_key(|((_, pi), (_, pj))| (**pi - **pj).norm_l2_sq())
        .scan(
            ((0..boxes.len()).collect_vec(), boxes.len()),
            |(assoc, n_grp), ((i, pi), (j, pj))| {
                if update_network_associations(assoc, i, j) == UpdateOutcome::NetworksConnected {
                    *n_grp -= 1;
                }
                Some(((*pi, *pj), *n_grp))
            },
        )
        .find(|(_, n_grp)| *n_grp == 1)
        .expect("at least 2 junction boxes should be present")
        .0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UpdateOutcome {
    AlreadyConnected,
    NetworksConnected,
}

fn update_network_associations(assoc: &mut [usize], i: usize, j: usize) -> UpdateOutcome {
    let lhs = assoc[i];
    let rhs = assoc[j];

    if lhs != rhs {
        assoc
            .iter_mut()
            .filter(|x| **x == rhs)
            .for_each(|x| *x = lhs);

        UpdateOutcome::NetworksConnected
    } else {
        UpdateOutcome::AlreadyConnected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_BOXES: &[Vector<i64, 3>] = &[
        Vector([162, 817, 812]),
        Vector([57, 618, 57]),
        Vector([906, 360, 560]),
        Vector([592, 479, 940]),
        Vector([352, 342, 300]),
        Vector([466, 668, 158]),
        Vector([542, 29, 236]),
        Vector([431, 825, 988]),
        Vector([739, 650, 466]),
        Vector([52, 470, 668]),
        Vector([216, 146, 977]),
        Vector([819, 987, 18]),
        Vector([117, 168, 530]),
        Vector([805, 96, 715]),
        Vector([346, 949, 466]),
        Vector([970, 615, 88]),
        Vector([941, 993, 340]),
        Vector([862, 61, 35]),
        Vector([984, 92, 344]),
        Vector([425, 690, 689]),
    ];

    #[test]
    fn example_network_counts() {
        itertools::assert_equal(
            networks(EXAMPLE_BOXES, 10)
                .iter()
                .counts()
                .into_values()
                .sorted()
                .rev(),
            [5, 4, 2, 2, 1, 1, 1, 1, 1, 1, 1],
        );
    }

    #[test]
    fn example_final_connection() {
        assert_eq!(
            final_connection(EXAMPLE_BOXES),
            (Vector([216, 146, 977]), Vector([117, 168, 530]))
        )
    }
}
