use std::collections::HashSet;

use aoc_companion::prelude::*;
use aoc_utils::{
    array,
    geometry::{Point, parse_ascii_map},
    linalg::Vector,
};
use ndarray::ShapeError;

type Map = ndarray::Array2<u8>;

pub(crate) struct Door {
    map: Map,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ShapeError> {
        Ok(Self {
            map: parse_ascii_map(input)?,
        })
    }

    fn part1(&self) -> usize {
        fencing_price(&self.map)
    }

    fn part2(&self) -> usize {
        discounted_price(&self.map)
    }
}

fn segment(map: &Map) -> Vec<HashSet<Vector<usize, 2>>> {
    let mut not_visited: HashSet<_> = map
        .indexed_iter()
        .map(|((x, y), _)| Vector([x, y]))
        .collect();

    std::iter::from_fn(move || {
        not_visited.iter().next().cloned().map(|seed| {
            not_visited.remove(&seed);
            let mut cluster = HashSet::from([seed]);
            let mut wave = HashSet::from([seed]);
            let crop = map[seed];
            while !wave.is_empty() {
                wave = wave
                    .iter()
                    .flat_map(|p| p.nearest_neighbors())
                    .filter(|p| map.get(*p) == Some(&crop))
                    .filter(|p| not_visited.remove(p))
                    .collect();
                cluster.extend(wave.iter().cloned());
            }
            cluster
        })
    })
    .collect()
}

fn area(cluster: &HashSet<Vector<usize, 2>>) -> usize {
    cluster.len()
}

fn perimeter(cluster: &HashSet<Vector<usize, 2>>) -> usize {
    cluster
        .iter()
        .flat_map(|p| p.try_cast_as::<isize>().unwrap().nearest_neighbors())
        .filter(|p| !p.try_cast_as::<usize>().is_ok_and(|q| cluster.contains(&q)))
        .count()
}

fn edges(cluster: &HashSet<Vector<usize, 2>>) -> usize {
    // The solution is based on the Euler characteristic (vertices - edges + faces = 1)
    // which in this case implies that the number of edges should always equal
    // the number of vertices since the cluster is one-faced by construction.
    // Counting vertices is easier because it is sufficient to consider the immediate
    // neighborhood of each point in the cluster to determine whether it is a corner.
    let total_vertex_value: usize = cluster
        .iter()
        .map(|p| p.try_cast_as::<isize>().unwrap())
        .map(|p| {
            array::from_iter_exact::<bool, 8>(
                p.neighbors()
                    .map(|q| q.try_cast_as::<usize>().is_ok_and(|q| cluster.contains(&q))),
            )
            .expect("point should have exactly eight neighbors")
        })
        .map(vertex_value)
        .sum();

    assert!(
        total_vertex_value.is_multiple_of(3),
        "corner value should be divisible by three; got {total_vertex_value}"
    );

    total_vertex_value / 3
}

fn vertex_value(neighbors: [bool; 8]) -> usize {
    // Each point in the cluster can contribute 0 to 4 corners to the overall shape
    // (4 being possible only if the cluster is just a single point).
    // Due to triple counting, the value returned by this function is trice the actual
    // (fractional) contribution of this point. The total "vertex value" over all points
    // in the cluster then has to be divisible by three to produce the integer number of
    // vertices.
    // `vertex_contributions` yields the contribution of each of the four potential
    // corners of the cluster "pixel", considering the neighbor diagonally across and
    // two adjacent neighbors which share the edge segments which meet in the potential
    // corner.
    // `neighbors` assumes the order which the 2D `neighbors()` impls use.
    vertex_contribution(neighbors[7], neighbors[0], neighbors[6])
        + vertex_contribution(neighbors[1], neighbors[2], neighbors[0])
        + vertex_contribution(neighbors[3], neighbors[4], neighbors[2])
        + vertex_contribution(neighbors[5], neighbors[6], neighbors[4])
}

fn vertex_contribution(diag: bool, adj1: bool, adj2: bool) -> usize {
    match (diag, adj1, adj2) {
        (true, true, true) => 0,
        (true, true, false) => 1,
        (true, false, true) => 1,
        (true, false, false) => 3,
        (false, true, true) => 1,
        (false, true, false) => 0,
        (false, false, true) => 0,
        (false, false, false) => 3,
    }
}

fn fencing_price(map: &Map) -> usize {
    segment(map)
        .iter()
        .map(|cluster| area(cluster) * perimeter(cluster))
        .sum()
}

fn discounted_price(map: &Map) -> usize {
    segment(map)
        .iter()
        .map(|cluster| area(cluster) * edges(cluster))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIRST_EXAMPLE_INPUT: &str = "\
AAAA
BBCD
BBCC
EEEC";

    const SECOND_EXAMPLE_INPUT: &str = "\
OOOOO
OXOXO
OOOOO
OXOXO
OOOOO";

    const THIRD_EXAMPLE_INPUT: &str = "\
RRRRIICCFF
RRRRIICCCF
VVRRRCCFFF
VVRCCCJFFF
VVVVCJJCFE
VVIVCCJJEE
VVIIICJJEE
MIIIIIJJEE
MIIISIJEEE
MMMISSJEEE";

    const E_SHAPED_INPUT: &str = "\
EEEEE
EXXXX
EEEEE
EXXXX
EEEEE";

    const TOUCHING_HOLE_INPUT: &str = "\
AAAAAA
AAABBA
AAABBA
ABBAAA
ABBAAA
AAAAAA";

    #[test]
    fn single_point_edges() {
        let cluster = HashSet::from([Vector([0, 0])]);
        assert_eq!(edges(&cluster), 4);
    }

    #[test]
    fn square_edges() {
        let cluster = HashSet::from([
            Vector([0, 0]),
            Vector([0, 1]),
            Vector([1, 0]),
            Vector([1, 1]),
        ]);
        assert_eq!(edges(&cluster), 4);
    }

    #[test]
    fn l_shaped_cluster_edges() {
        let cluster = HashSet::from([Vector([0, 0]), Vector([0, 1]), Vector([1, 0])]);
        assert_eq!(edges(&cluster), 6);
    }

    #[test]
    fn o_shaped_cluster_edges() {
        let cluster = HashSet::from([
            Vector([0, 0]),
            Vector([0, 1]),
            Vector([0, 2]),
            Vector([1, 2]),
            Vector([2, 2]),
            Vector([2, 1]),
            Vector([2, 0]),
            Vector([1, 0]),
        ]);
        assert_eq!(edges(&cluster), 8);
    }

    #[test]
    fn c_shaped_cluster_edges() {
        let cluster = HashSet::from([
            Vector([2, 1]),
            Vector([3, 1]),
            Vector([3, 2]),
            Vector([3, 3]),
            Vector([2, 3]),
        ]);
        assert_eq!(edges(&cluster), 8);
    }

    #[test]
    fn e_shaped_cluster_edges() {
        let cluster = HashSet::from([
            Vector([0, 0]),
            Vector([1, 0]),
            Vector([2, 0]),
            Vector([3, 0]),
            Vector([4, 0]),
            Vector([0, 1]),
            Vector([0, 2]),
            Vector([1, 2]),
            Vector([2, 2]),
            Vector([3, 2]),
            Vector([4, 2]),
            Vector([0, 3]),
            Vector([0, 4]),
            Vector([1, 4]),
            Vector([2, 4]),
            Vector([3, 4]),
            Vector([4, 4]),
        ]);
        assert_eq!(edges(&cluster), 12);
    }

    #[test]
    fn fencing_price_using_perimeter() {
        assert_eq!(
            fencing_price(&parse_ascii_map(FIRST_EXAMPLE_INPUT).unwrap()),
            140
        );
        assert_eq!(
            fencing_price(&parse_ascii_map(SECOND_EXAMPLE_INPUT).unwrap()),
            772
        );
        assert_eq!(
            fencing_price(&parse_ascii_map(THIRD_EXAMPLE_INPUT).unwrap()),
            1930
        );
    }

    #[test]
    fn fencing_price_with_bulk_discount() {
        assert_eq!(
            discounted_price(&parse_ascii_map(FIRST_EXAMPLE_INPUT).unwrap()),
            80
        );
        assert_eq!(
            discounted_price(&parse_ascii_map(SECOND_EXAMPLE_INPUT).unwrap()),
            436
        );
        assert_eq!(
            discounted_price(&parse_ascii_map(E_SHAPED_INPUT).unwrap()),
            236
        );
        assert_eq!(
            discounted_price(&parse_ascii_map(TOUCHING_HOLE_INPUT).unwrap()),
            368
        );
    }
}
