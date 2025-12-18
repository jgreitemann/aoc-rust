use std::{array, collections::HashSet, ops::RangeInclusive};

use aoc_companion::prelude::*;
use aoc_utils::{geometry::Point, linalg::Vector};
use itertools::Itertools;

pub(crate) struct Door {
    seats: HashSet<Vector<isize, 2>>,
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
                    .map(move |(col, _)| Vector([row as isize, col as isize]))
            })
            .collect();
        Door { seats }
    }

    fn part1(&self) -> usize {
        fixed_point_occupancy(&self.seats, &DirectNeighborSeatPolicy).len()
    }

    fn part2(&self) -> usize {
        fixed_point_occupancy(&self.seats, &SightlineSeatPolicy::new(&self.seats)).len()
    }
}

trait SeatPolicy {
    const THRESHOLD: usize;
    fn neighbors(&self, seat: Vector<isize, 2>) -> impl Iterator<Item = Vector<isize, 2>>;
}

struct DirectNeighborSeatPolicy;

impl SeatPolicy for DirectNeighborSeatPolicy {
    const THRESHOLD: usize = 4;

    fn neighbors(&self, seat: Vector<isize, 2>) -> impl Iterator<Item = Vector<isize, 2>> {
        seat.neighbors()
    }
}

struct SightlineSeatPolicy<'s> {
    seats: &'s HashSet<Vector<isize, 2>>,
    bounds: [RangeInclusive<isize>; 2],
}

impl<'s> SightlineSeatPolicy<'s> {
    fn new(seats: &'s HashSet<Vector<isize, 2>>) -> Self {
        Self {
            seats,
            bounds: array::from_fn(|i| 0..=seats.iter().map(|s| s[i]).max().unwrap_or(0)),
        }
    }
}

impl SeatPolicy for SightlineSeatPolicy<'_> {
    const THRESHOLD: usize = 5;

    fn neighbors(&self, seat: Vector<isize, 2>) -> impl Iterator<Item = Vector<isize, 2>> {
        seat.neighbors().filter_map(move |neighbor| {
            let dir = neighbor - seat;
            (1..)
                .map(|n| seat + dir * n)
                .take_while(|s| s.in_bounds(&self.bounds))
                .find(|s| self.seats.contains(s))
        })
    }
}

fn evolve<P: SeatPolicy>(
    occupied: &HashSet<Vector<isize, 2>>,
    seats: &HashSet<Vector<isize, 2>>,
    policy: &P,
) -> HashSet<Vector<isize, 2>> {
    let empty = seats.difference(occupied);
    empty
        .copied()
        .filter(|seat| policy.neighbors(*seat).all(|n| !occupied.contains(&n)))
        .chain(occupied.iter().copied().filter(|seat| {
            policy
                .neighbors(*seat)
                .filter(|n| occupied.contains(n))
                .count()
                < P::THRESHOLD
        }))
        .collect()
}

fn fixed_point_occupancy(
    seats: &HashSet<Vector<isize, 2>>,
    policy: &impl SeatPolicy,
) -> HashSet<Vector<isize, 2>> {
    itertools::iterate(HashSet::new(), |prev| evolve(prev, seats, policy))
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

    const EXAMPLE_SEATS: [Vector<isize, 2>; 71] = [
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
        assert_eq!(
            evolve(&HashSet::new(), &seats, &DirectNeighborSeatPolicy),
            seats
        );
    }

    #[test]
    fn number_of_occupied_seats_evolves() {
        let seats = HashSet::from(EXAMPLE_SEATS);
        let mut occupied = HashSet::new();
        occupied = evolve(&occupied, &seats, &DirectNeighborSeatPolicy);
        assert_eq!(occupied.len(), EXAMPLE_SEATS.len());
        occupied = evolve(&occupied, &seats, &DirectNeighborSeatPolicy);
        assert_eq!(occupied.len(), 20);
        occupied = evolve(&occupied, &seats, &DirectNeighborSeatPolicy);
        assert_eq!(occupied.len(), 51);
        occupied = evolve(&occupied, &seats, &DirectNeighborSeatPolicy);
        assert_eq!(occupied.len(), 30);
        occupied = evolve(&occupied, &seats, &DirectNeighborSeatPolicy);
        assert_eq!(occupied.len(), 37);
        occupied = evolve(&occupied, &seats, &DirectNeighborSeatPolicy);
        assert_eq!(occupied.len(), 37);
    }

    #[test]
    fn find_fixed_point_occupancy() {
        let seats = HashSet::from(EXAMPLE_SEATS);
        assert_eq!(
            fixed_point_occupancy(&seats, &DirectNeighborSeatPolicy).len(),
            37
        );
        assert_eq!(
            fixed_point_occupancy(&seats, &SightlineSeatPolicy::new(&seats)).len(),
            26
        );
    }
}
