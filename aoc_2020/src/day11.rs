use std::collections::HashSet;

use aoc_companion::prelude::*;
use aoc_utils::{geometry::Point, linalg::Vector};
use itertools::Itertools;

pub(crate) struct Door {
    seats: HashSet<Vector<usize, 2>>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Self {
        let seats = input
            .lines()
            .enumerate()
            .flat_map(|(row, line)| {
                line.bytes()
                    .enumerate()
                    .filter(|&(_, b)| b == b'L')
                    .map(move |(col, _)| Vector([row, col]))
            })
            .collect();
        Door { seats }
    }

    fn part1(&self) -> usize {
        fixed_point_occupancy(&self.seats).len()
    }
}

fn evolve(
    occupied: &HashSet<Vector<usize, 2>>,
    seats: &HashSet<Vector<usize, 2>>,
) -> HashSet<Vector<usize, 2>> {
    let empty = seats.difference(occupied);
    empty
        .copied()
        .filter(|seat| seat.neighbors().all(|n| !occupied.contains(&n)))
        .chain(
            occupied
                .iter()
                .copied()
                .filter(|seat| seat.neighbors().filter(|n| occupied.contains(n)).count() < 4),
        )
        .collect()
}

fn fixed_point_occupancy(seats: &HashSet<Vector<usize, 2>>) -> HashSet<Vector<usize, 2>> {
    itertools::iterate(HashSet::new(), |prev| evolve(prev, seats))
        .tuple_windows()
        .find(|(lhs, rhs)| lhs == rhs)
        .unwrap()
        .1
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "\
L.LL.LL.LL
LLLLLLL.LL
L.L.L..L..
LLLL.LL.LL
L.LL.LL.LL
L.LLLLL.LL
..L.L.....
LLLLLLLLLL
L.LLLLLL.L
L.LLLLL.LL";

    const EXAMPLE_SEATS: [Vector<usize, 2>; 71] = [
        Vector([0, 0]),
        Vector([0, 2]),
        Vector([0, 3]),
        Vector([0, 5]),
        Vector([0, 6]),
        Vector([0, 8]),
        Vector([0, 9]),
        Vector([1, 0]),
        Vector([1, 1]),
        Vector([1, 2]),
        Vector([1, 3]),
        Vector([1, 4]),
        Vector([1, 5]),
        Vector([1, 6]),
        Vector([1, 8]),
        Vector([1, 9]),
        Vector([2, 0]),
        Vector([2, 2]),
        Vector([2, 4]),
        Vector([2, 7]),
        Vector([3, 0]),
        Vector([3, 1]),
        Vector([3, 2]),
        Vector([3, 3]),
        Vector([3, 5]),
        Vector([3, 6]),
        Vector([3, 8]),
        Vector([3, 9]),
        Vector([4, 0]),
        Vector([4, 2]),
        Vector([4, 3]),
        Vector([4, 5]),
        Vector([4, 6]),
        Vector([4, 8]),
        Vector([4, 9]),
        Vector([5, 0]),
        Vector([5, 2]),
        Vector([5, 3]),
        Vector([5, 4]),
        Vector([5, 5]),
        Vector([5, 6]),
        Vector([5, 8]),
        Vector([5, 9]),
        Vector([6, 2]),
        Vector([6, 4]),
        Vector([7, 0]),
        Vector([7, 1]),
        Vector([7, 2]),
        Vector([7, 3]),
        Vector([7, 4]),
        Vector([7, 5]),
        Vector([7, 6]),
        Vector([7, 7]),
        Vector([7, 8]),
        Vector([7, 9]),
        Vector([8, 0]),
        Vector([8, 2]),
        Vector([8, 3]),
        Vector([8, 4]),
        Vector([8, 5]),
        Vector([8, 6]),
        Vector([8, 7]),
        Vector([8, 9]),
        Vector([9, 0]),
        Vector([9, 2]),
        Vector([9, 3]),
        Vector([9, 4]),
        Vector([9, 5]),
        Vector([9, 6]),
        Vector([9, 8]),
        Vector([9, 9]),
    ];

    #[test]
    fn parse_seat_map() {
        let door = Door::parse(EXAMPLE_INPUT);
        assert_eq!(door.seats, HashSet::from(EXAMPLE_SEATS));
    }

    #[test]
    fn all_seats_occupied_after_first_round() {
        let seats = HashSet::from(EXAMPLE_SEATS);
        assert_eq!(evolve(&HashSet::new(), &seats), seats);
    }

    #[test]
    fn number_of_occupied_seats_evolves() {
        let seats = HashSet::from(EXAMPLE_SEATS);
        let mut occupied = HashSet::new();
        occupied = evolve(&occupied, &seats);
        assert_eq!(occupied.len(), EXAMPLE_SEATS.len());
        occupied = evolve(&occupied, &seats);
        assert_eq!(occupied.len(), 20);
        occupied = evolve(&occupied, &seats);
        assert_eq!(occupied.len(), 51);
        occupied = evolve(&occupied, &seats);
        assert_eq!(occupied.len(), 30);
        occupied = evolve(&occupied, &seats);
        assert_eq!(occupied.len(), 37);
        occupied = evolve(&occupied, &seats);
        assert_eq!(occupied.len(), 37);
    }

    #[test]
    fn find_fixed_point_occupancy() {
        assert_eq!(
            fixed_point_occupancy(&HashSet::from(EXAMPLE_SEATS)).len(),
            37
        );
    }
}
