use std::collections::BTreeSet;

use aoc_companion::prelude::*;
use aoc_utils::{
    iter::AtMostThree,
    linalg::{ParseVectorError, Vector},
};
use itertools::Itertools as _;
use rayon::prelude::*;

pub(crate) struct Door {
    tiles: Vec<Vector<i32, 2>>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseVectorError<std::num::ParseIntError>> {
        input
            .lines()
            .map(str::parse)
            .try_collect()
            .map(|tiles| Door { tiles })
    }

    fn part1(&self) -> u64 {
        max_area(&self.tiles)
    }

    fn part2(&self) -> u64 {
        max_area_contained(&self.tiles)
    }
}

fn max_area(tiles: &[Vector<i32, 2>]) -> u64 {
    tiles
        .iter()
        .copied()
        .tuple_combinations()
        .map(rect_area)
        .max()
        .expect("at least two tiles are required")
}

fn max_area_contained(tiles: &[Vector<i32, 2>]) -> u64 {
    let mapping_x = find_mapping(tiles.iter().map(|v| v[0]));
    let mapping_y = find_mapping(tiles.iter().map(|v| v[1]));
    let mapping_slices = (mapping_x.as_slice(), mapping_y.as_slice());

    tiles
        .iter()
        .copied()
        .tuple_combinations()
        .sorted_by_key(|rect| rect_area(*rect))
        .collect_vec()
        .into_par_iter()
        .rev()
        .find_first(|rect| {
            rect_points(
                deflate(rect.0, mapping_slices),
                deflate(rect.1, mapping_slices),
            )
            .all(|p| is_point_in_polygon(inflate(p, mapping_slices), tiles))
        })
        .inspect(|rect| {
            dbg!(rect);
        })
        .map(rect_area)
        .expect("at least two tiles are required")
}

fn rect_area((a, b): (Vector<i32, 2>, Vector<i32, 2>)) -> u64 {
    let diff = b - a;
    (diff[0].unsigned_abs() as u64 + 1) * (diff[1].unsigned_abs() as u64 + 1)
}

fn rect_points(a: Vector<usize, 2>, b: Vector<usize, 2>) -> impl Iterator<Item = Vector<usize, 2>> {
    (a[0].min(b[0])..=a[0].max(b[0]))
        .cartesian_product(a[1].min(b[1])..=a[1].max(b[1]))
        .map(|(x, y)| Vector([x, y]))
}

fn is_point_in_polygon(point: Vector<i32, 2>, polygon: &[Vector<i32, 2>]) -> bool {
    is_on_polygon_boundary(point, polygon)
        || polygon
            .iter()
            .copied()
            .circular_tuple_windows()
            .filter(|segment| is_right_of_intersection(point, *segment))
            .count()
            % 2
            == 1
}

fn is_on_polygon_boundary(point: Vector<i32, 2>, polygon: &[Vector<i32, 2>]) -> bool {
    polygon
        .iter()
        .copied()
        .circular_tuple_windows()
        .any(|segment: (_, _)| {
            if segment.0[1] == segment.1[1] {
                point[1] == segment.0[1]
                    && (segment.0[0].min(segment.1[0])..=segment.0[0].max(segment.1[0]))
                        .contains(&point[0])
            } else if segment.0[0] == segment.1[0] {
                point[0] == segment.0[0]
                    && (segment.0[1].min(segment.1[1])..=segment.0[1].max(segment.1[1]))
                        .contains(&point[1])
            } else {
                panic!("non-axis-parallel lines are not supported");
            }
        })
}

fn is_right_of_intersection(
    point: Vector<i32, 2>,
    segment: (Vector<i32, 2>, Vector<i32, 2>),
) -> bool {
    if segment.0[1] == segment.1[1] {
        false
    } else if segment.0[0] == segment.1[0] {
        point[0] > segment.0[0]
            && ((segment.0[1]..segment.1[1]).contains(&point[1])
                || (segment.1[1]..segment.0[1]).contains(&point[1]))
    } else {
        panic!("non-axis-parallel lines are not supported");
    }
}

fn find_mapping(coords: impl IntoIterator<Item = i32>) -> Vec<i32> {
    BTreeSet::from_iter(coords)
        .iter()
        .circular_tuple_windows()
        .flat_map(|(&a, &b)| {
            if a + 1 == b {
                AtMostThree::two(a, b)
            } else {
                AtMostThree::three(a, a + 1, b)
            }
        })
        .collect()
}

fn deflate(v: Vector<i32, 2>, mapping: (&[i32], &[i32])) -> Vector<usize, 2> {
    Vector([
        mapping.0.binary_search(&v[0]).unwrap(),
        mapping.1.binary_search(&v[1]).unwrap(),
    ])
}

fn inflate(v: Vector<usize, 2>, mapping: (&[i32], &[i32])) -> Vector<i32, 2> {
    Vector([mapping.0[v[0]], mapping.1[v[1]]])
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use aoc_utils::geometry::Point;

    use super::*;

    const EXAMPLE_TILES: &[Vector<i32, 2>] = &[
        Vector([7, 1]),
        Vector([11, 1]),
        Vector([11, 7]),
        Vector([9, 7]),
        Vector([9, 5]),
        Vector([2, 5]),
        Vector([2, 3]),
        Vector([7, 3]),
    ];

    const EXAMPLE_MAP: &str = "\
..............
.......#XXX#..
.......XXXXX..
..#XXXX#XXXX..
..XXXXXXXXXX..
..#XXXXXX#XX..
.........XXX..
.........#X#..
..............";

    const PATHOLOGICAL_TILES: &[Vector<i32, 2>] = &[
        Vector([1, 1]),
        Vector([5, 1]),
        Vector([5, 6]),
        Vector([8, 6]),
        Vector([8, 1]),
        Vector([10, 1]),
        Vector([10, 5]),
        Vector([12, 5]),
        Vector([12, 7]),
        Vector([3, 7]),
        Vector([3, 3]),
        Vector([1, 3]),
    ];

    const PATHOLOGICAL_MAP: &str = "\
..............
.#XXX#..#X#...
.XXXXX..XXX...
.#X#XX..XXX...
...XXX..XXX...
...XXX..XX#X#.
...XX#XX#XXXX.
...#XXXXXXXX#.
..............";

    const NARROW_TILES: &[Vector<i32, 2>] = &[
        Vector([1, 1]),
        Vector([7, 1]),
        Vector([7, 3]),
        Vector([3, 3]),
        Vector([3, 4]),
        Vector([7, 4]),
        Vector([7, 6]),
        Vector([1, 6]),
    ];

    const NARROW_MAP: &str = "\
.........
.#XXXXX#.
.XXXXXXX.
.XX#XXX#.
.XX#XXX#.
.XXXXXXX.
.#XXXXX#.
.........";

    const ANVIL_TILES: &[Vector<i32, 2>] = &[
        Vector([1, 1]),
        Vector([14, 1]),
        Vector([14, 12]),
        Vector([1, 12]),
        Vector([1, 7]),
        Vector([7, 7]),
        Vector([7, 8]),
        Vector([2, 8]),
        Vector([2, 11]),
        Vector([13, 11]),
        Vector([13, 8]),
        Vector([8, 8]),
        Vector([8, 5]),
        Vector([13, 5]),
        Vector([13, 2]),
        Vector([2, 2]),
        Vector([2, 5]),
        Vector([7, 5]),
        Vector([7, 6]),
        Vector([1, 6]),
    ];

    const ANVIL_MAP: &str = "\
................
.#XXXXXXXXXXXX#.
.X#XXXXXXXXXX#X.
.XX..........XX.
.XX..........XX.
.X#XXXX##XXXX#X.
.#XXXXX#XXXXXXX.
.#XXXXX#XXXXXXX.
.X#XXXX##XXXX#X.
.XX..........XX.
.XX..........XX.
.X#XXXXXXXXXX#X.
.#XXXXXXXXXXXX#.
................";

    #[test]
    fn max_area_spanned_by_tiles() {
        assert_eq!(max_area(EXAMPLE_TILES), 50);
    }

    #[test]
    fn max_area_contained_by_tiles() {
        assert_eq!(max_area_contained(EXAMPLE_TILES), 24);
        assert_eq!(max_area_contained(PATHOLOGICAL_TILES), 21);
        assert_eq!(max_area_contained(NARROW_TILES), 42);
        assert_eq!(max_area_contained(ANVIL_TILES), 48);
    }

    fn test_point_in_polygon(tiles: &[Vector<i32, 2>], map: &str) {
        let polygon_points: HashSet<Vector<i32, 2>> = aoc_utils::geometry::parse_ascii_map(map)
            .unwrap()
            .indexed_iter()
            .filter(|(_, b)| **b != b'.')
            .map(|((y, x), _)| Vector([x as i32, y as i32]))
            .collect();
        let polygon_neighbors: HashSet<Vector<i32, 2>> = polygon_points
            .iter()
            .flat_map(|p| p.neighbors())
            .filter(|p| !polygon_points.contains(p))
            .collect();

        for p in polygon_points {
            assert!(
                is_point_in_polygon(p, tiles),
                "{p:?} is in the polygon, but is_point_in_polygon said otherwise"
            );
        }

        for p in polygon_neighbors {
            assert!(
                !is_point_in_polygon(p, tiles),
                "{p:?} is NOT in the polygon, but is_point_in_polygon said otherwise"
            );
        }
    }

    #[test]
    fn point_in_example_polygon() {
        test_point_in_polygon(EXAMPLE_TILES, EXAMPLE_MAP);
    }

    #[test]
    fn point_in_pathological_polygon() {
        test_point_in_polygon(PATHOLOGICAL_TILES, PATHOLOGICAL_MAP);
    }

    #[test]
    fn point_in_narrow_polygon() {
        test_point_in_polygon(NARROW_TILES, NARROW_MAP);
    }

    #[test]
    fn point_in_anvil_polygon() {
        test_point_in_polygon(ANVIL_TILES, ANVIL_MAP);
    }
}
