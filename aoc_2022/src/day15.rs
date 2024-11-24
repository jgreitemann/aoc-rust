use aoc_companion::prelude::*;
use aoc_utils::linalg::Vector;

use itertools::Itertools;
use tap::Tap;
use thiserror::Error;

use std::collections::HashSet;
use std::num::ParseIntError;
use std::ops::RangeInclusive;

const LINE_Y: isize = 2000000;

pub struct Door {
    sensors: Vec<SensorData>,
}

impl ParseInput<'_> for Door {
    type Error = ParseIntError;

    fn parse(input: &str) -> Result<Self, Self::Error> {
        parse_input(input).map(|sensors| Self { sensors })
    }
}

impl Part1 for Door {
    type Output = usize;
    type Error = std::convert::Infallible;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        Ok(coverage_on_line(&self.sensors, LINE_Y).size()
            - number_of_beacons_on_line(&self.sensors, LINE_Y))
    }
}

impl Part2 for Door {
    type Output = isize;
    type Error = RuntimeError;

    fn part2(&self) -> Result<Self::Output, Self::Error> {
        const N: isize = 4000000;
        find_distress_beacon_in_bounds(&self.sensors, 0..=N)
            .ok_or(RuntimeError::DistressBeaconNotFound)
            .map(|Vector([x, y])| x * N + y)
    }
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("No distress beacon found in bounds")]
    DistressBeaconNotFound,
}

type Position = Vector<isize, 2>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct SensorData {
    sensor_pos: Position,
    closest_beacon_pos: Position,
}

impl SensorData {
    fn range(&self) -> isize {
        (self.sensor_pos - self.closest_beacon_pos).norm_l1()
    }

    fn line_covering(&self, line_y: isize) -> RangeInclusive<isize> {
        let range = self.range();
        let div = (self.sensor_pos[1] - line_y).abs();
        (self.sensor_pos[0] - range + div)..=(self.sensor_pos[0] + range - div)
    }

    fn covers(&self, p: &Position) -> bool {
        (self.sensor_pos - *p).norm_l1() <= self.range()
    }

    fn bordering_positions(&self) -> impl Iterator<Item = Position> {
        let radius = self.range() + 1;
        let north = self.sensor_pos + Vector([0, radius]);
        let west = self.sensor_pos + Vector([-radius, 0]);
        let south = self.sensor_pos + Vector([0, -radius]);
        let east = self.sensor_pos + Vector([radius, 0]);
        (0..radius)
            .map(move |i| north + Vector([-1, -1]) * i)
            .chain((0..radius).map(move |i| west + Vector([1, -1]) * i))
            .chain((0..radius).map(move |i| south + Vector([1, 1]) * i))
            .chain((0..radius).map(move |i| east + Vector([-1, 1]) * i))
    }
}

fn parse_input(input: &str) -> Result<Vec<SensorData>, ParseIntError> {
    let re = regex::Regex::new(r"Sensor at x=(?P<sx>-?\d+), y=(?P<sy>-?\d+): closest beacon is at x=(?P<bx>-?\d+), y=(?P<by>-?\d+)").unwrap();
    re.captures_iter(input)
        .map(|capt| {
            Ok(SensorData {
                sensor_pos: Vector([capt["sx"].parse()?, capt["sy"].parse()?]),
                closest_beacon_pos: Vector([capt["bx"].parse()?, capt["by"].parse()?]),
            })
        })
        .try_collect()
}

#[derive(Debug, PartialEq, Eq)]
struct Coverage<C: Covering> {
    positive: Vec<C>,
    negative: Vec<C>,
}

impl<C: Covering> Coverage<C> {
    fn new() -> Self {
        Self {
            positive: Vec::new(),
            negative: Vec::new(),
        }
    }

    fn add(&mut self, new: C) {
        let pos_intersections: Vec<_> = self
            .positive
            .iter()
            .flat_map(|p| p.intersect(&new))
            .collect();
        let neg_intersections: Vec<_> = self
            .negative
            .iter()
            .flat_map(|p| p.intersect(&new))
            .collect();
        self.positive.push(new);
        self.negative.extend(pos_intersections);
        self.positive.extend(neg_intersections);
    }

    fn size(&self) -> usize {
        self.positive.iter().map(Covering::size).sum::<usize>()
            - self.negative.iter().map(Covering::size).sum::<usize>()
    }
}

trait Covering: Sized {
    fn intersect(&self, other: &Self) -> Option<Self>;
    fn size(&self) -> usize;
}

impl Covering for RangeInclusive<isize> {
    fn intersect(&self, other: &Self) -> Option<Self> {
        let range = (*self.start().max(other.start()))..=(*self.end().min(other.end()));
        (!range.is_empty()).then_some(range)
    }

    fn size(&self) -> usize {
        if self.is_empty() {
            0
        } else {
            (self.end() + 1 - self.start()) as usize
        }
    }
}

fn coverage_on_line(sensors: &[SensorData], line_y: isize) -> Coverage<RangeInclusive<isize>> {
    sensors.iter().fold(Coverage::new(), |cov, sensor| {
        cov.tap_mut(|c| c.add(sensor.line_covering(line_y)))
    })
}

fn number_of_beacons_on_line(sensors: &[SensorData], line_y: isize) -> usize {
    let beacons_on_line: HashSet<_> = sensors
        .iter()
        .map(|s| s.closest_beacon_pos)
        .filter(|&Vector([_, y])| y == line_y)
        .collect();
    beacons_on_line.len()
}

fn find_distress_beacon_in_bounds(
    sensors: &[SensorData],
    bounds: RangeInclusive<isize>,
) -> Option<Position> {
    sensors
        .iter()
        .flat_map(|s| s.bordering_positions())
        .filter(|b| bounds.contains(&b[0]) && bounds.contains(&b[1]))
        .find(|b| sensors.iter().all(|s| !s.covers(b)))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "\
Sensor at x=2, y=18: closest beacon is at x=-2, y=15
Sensor at x=9, y=16: closest beacon is at x=10, y=16
Sensor at x=13, y=2: closest beacon is at x=15, y=3
Sensor at x=12, y=14: closest beacon is at x=10, y=16
Sensor at x=10, y=20: closest beacon is at x=10, y=16
Sensor at x=14, y=17: closest beacon is at x=10, y=16
Sensor at x=8, y=7: closest beacon is at x=2, y=10
Sensor at x=2, y=0: closest beacon is at x=2, y=10
Sensor at x=0, y=11: closest beacon is at x=2, y=10
Sensor at x=20, y=14: closest beacon is at x=25, y=17
Sensor at x=17, y=20: closest beacon is at x=21, y=22
Sensor at x=16, y=7: closest beacon is at x=15, y=3
Sensor at x=14, y=3: closest beacon is at x=15, y=3
Sensor at x=20, y=1: closest beacon is at x=15, y=3";

    const EXAMPLE_SENSOR_DATA: &[SensorData] = &[
        SensorData {
            sensor_pos: Vector([2, 18]),
            closest_beacon_pos: Vector([-2, 15]),
        },
        SensorData {
            sensor_pos: Vector([9, 16]),
            closest_beacon_pos: Vector([10, 16]),
        },
        SensorData {
            sensor_pos: Vector([13, 2]),
            closest_beacon_pos: Vector([15, 3]),
        },
        SensorData {
            sensor_pos: Vector([12, 14]),
            closest_beacon_pos: Vector([10, 16]),
        },
        SensorData {
            sensor_pos: Vector([10, 20]),
            closest_beacon_pos: Vector([10, 16]),
        },
        SensorData {
            sensor_pos: Vector([14, 17]),
            closest_beacon_pos: Vector([10, 16]),
        },
        SensorData {
            sensor_pos: Vector([8, 7]),
            closest_beacon_pos: Vector([2, 10]),
        },
        SensorData {
            sensor_pos: Vector([2, 0]),
            closest_beacon_pos: Vector([2, 10]),
        },
        SensorData {
            sensor_pos: Vector([0, 11]),
            closest_beacon_pos: Vector([2, 10]),
        },
        SensorData {
            sensor_pos: Vector([20, 14]),
            closest_beacon_pos: Vector([25, 17]),
        },
        SensorData {
            sensor_pos: Vector([17, 20]),
            closest_beacon_pos: Vector([21, 22]),
        },
        SensorData {
            sensor_pos: Vector([16, 7]),
            closest_beacon_pos: Vector([15, 3]),
        },
        SensorData {
            sensor_pos: Vector([14, 3]),
            closest_beacon_pos: Vector([15, 3]),
        },
        SensorData {
            sensor_pos: Vector([20, 1]),
            closest_beacon_pos: Vector([15, 3]),
        },
    ];

    #[test]
    fn sensor_data_is_parsed() {
        assert_eq!(parse_input(EXAMPLE_INPUT).unwrap(), EXAMPLE_SENSOR_DATA);
    }

    #[test]
    fn line_covering_is_accurate() {
        const SENSOR: SensorData = SensorData {
            sensor_pos: Vector([8, 7]),
            closest_beacon_pos: Vector([2, 10]),
        };

        assert_eq!(SENSOR.line_covering(0), 6..=10);
        assert_eq!(SENSOR.line_covering(7), -1..=17);
        assert_eq!(SENSOR.line_covering(10), 2..=14);
        assert_eq!(SENSOR.line_covering(-2), 8..=8);
        assert!(SENSOR.line_covering(-3).is_empty());
        assert!(SENSOR.line_covering(17).is_empty());
        assert!(SENSOR.line_covering(20).is_empty());
    }

    // 1 2 3 4 5 6 7 8
    // ---------
    //     -----------

    #[test]
    fn intersection_of_line_segments() {
        assert_eq!(Covering::intersect(&(1..=5), &(3..=8)), Some(3..=5));
        assert_eq!(Covering::intersect(&(1..=8), &(3..=5)), Some(3..=5));
        assert_eq!(Covering::intersect(&(1..=3), &(5..=8)), None);
    }

    #[test]
    #[allow(clippy::reversed_empty_ranges)]
    fn size_of_line_segments() {
        assert_eq!((1..=5).size(), 5);
        assert_eq!((3..=5).size(), 3);
        assert_eq!((5..=3).size(), 0);
        assert_eq!((3..=3).size(), 1);
    }

    #[test]
    fn size_of_line_coverage() {
        let mut coverage = Coverage::new();
        coverage.add(1..=5);
        assert_eq!(coverage.size(), 5);
        coverage.add(3..=8);
        assert_eq!(coverage.size(), 8);
        coverage.add(11..=15);
        assert_eq!(coverage.size(), 13);
        coverage.add(4..=4);
        assert_eq!(coverage.size(), 13);
    }

    #[test]
    fn total_line_coverage_is_calculated() {
        assert_eq!(coverage_on_line(EXAMPLE_SENSOR_DATA, 9).size(), 25);
        assert_eq!(coverage_on_line(EXAMPLE_SENSOR_DATA, 10).size(), 27);
        assert_eq!(coverage_on_line(EXAMPLE_SENSOR_DATA, 11).size(), 28);
    }

    #[test]
    fn number_of_beacons_on_line_is_found() {
        assert_eq!(number_of_beacons_on_line(EXAMPLE_SENSOR_DATA, 9), 0);
        assert_eq!(number_of_beacons_on_line(EXAMPLE_SENSOR_DATA, 10), 1);
        assert_eq!(number_of_beacons_on_line(EXAMPLE_SENSOR_DATA, 11), 0);
    }

    //     0123456
    //    0   x
    //    1  x#x
    //    2 x###x
    //    3x##S##x
    //    4 xB##x
    //    5  x#x
    //    6   x

    #[test]
    fn bordering_points_are_found() {
        let points = HashSet::from([
            Vector([3, 0]),
            Vector([2, 1]),
            Vector([4, 1]),
            Vector([1, 2]),
            Vector([5, 2]),
            Vector([0, 3]),
            Vector([6, 3]),
            Vector([1, 4]),
            Vector([5, 4]),
            Vector([2, 5]),
            Vector([4, 5]),
            Vector([3, 6]),
        ]);
        const TEST_SENSOR: SensorData = SensorData {
            sensor_pos: Vector([3, 3]),
            closest_beacon_pos: Vector([2, 4]),
        };
        assert_eq!(
            TEST_SENSOR.bordering_positions().collect::<HashSet<_>>(),
            points
        );
    }

    #[test]
    fn distress_beacon_position_is_found() {
        assert_eq!(
            find_distress_beacon_in_bounds(EXAMPLE_SENSOR_DATA, 0..=20),
            Some(Vector([14, 11]))
        );
    }
}
