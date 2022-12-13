use aoc_companion::prelude::*;

use itertools::Itertools;
use thiserror::Error;

use std::cmp::Ordering;
use std::str::FromStr;

pub struct Door {
    pairs: Vec<(PacketData, PacketData)>,
}

impl ParseInput<'_> for Door {
    type Error = ParseError;

    fn parse(input: &str) -> Result<Self, Self::Error> {
        parse_input(input).map(|pairs| Self { pairs })
    }
}

impl Part1 for Door {
    type Output = usize;
    type Error = std::convert::Infallible;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        Ok(correctly_ordered_index_sum(&self.pairs))
    }
}

impl Part2 for Door {
    type Output = usize;
    type Error = std::convert::Infallible;

    fn part2(&self) -> Result<Self::Output, Self::Error> {
        Ok(decoder_key(&self.pairs))
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PacketData {
    Integer(u32),
    List(Vec<PacketData>),
}

impl FromStr for PacketData {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(list_str) = s.strip_prefix("[").and_then(|rest| rest.strip_suffix("]")) {
            let mut count = 0;
            list_str
                .split_terminator(|c| match c {
                    '[' => {
                        count += 1;
                        false
                    }
                    ']' => {
                        count -= 1;
                        false
                    }
                    ',' => count == 0,
                    _ => false,
                })
                .map(str::parse)
                .try_collect()
                .map(|list| PacketData::List(list))
        } else {
            Ok(PacketData::Integer(s.parse()?))
        }
    }
}

impl std::cmp::Ord for PacketData {
    fn cmp(&self, other: &Self) -> Ordering {
        use PacketData::*;
        match (self, other) {
            (Integer(x), Integer(y)) => std::cmp::Ord::cmp(x, y),
            (List(xs), List(ys)) => {
                match xs
                .iter()
                .zip(ys.iter())
                .fold(Ordering::Equal, |acc, (x, y)| match acc {
                    Ordering::Equal => std::cmp::Ord::cmp(x, y),
                    _ => acc,
                }) {
                    Ordering::Equal => std::cmp::Ord::cmp(&xs.len(), &ys.len()),
                    ord => ord,
                }
            }
            (list, Integer(y)) => std::cmp::Ord::cmp(list, &List(vec![Integer(*y)])),
            (Integer(x), list) => std::cmp::Ord::cmp(&List(vec![Integer(*x)]), list),
        }
    }
}

impl std::cmp::PartialOrd for PacketData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn parse_input(input: &str) -> Result<Vec<(PacketData, PacketData)>, ParseError> {
    input
        .split("\n\n")
        .map(|pair_str| pair_str.split_once("\n").expect("TODO"))
        .map(|(lhs, rhs)| Ok((lhs.parse()?, rhs.parse()?)))
        .try_collect()
}

fn correctly_ordered_index_sum(pairs: &[(PacketData, PacketData)]) -> usize {
    pairs
        .iter()
        .positions(|(lhs, rhs)| lhs < rhs)
        .map(|pos| pos + 1)
        .sum()
}

fn divider_packet(x: u32) -> PacketData {
    PacketData::List(vec![PacketData::List(vec![PacketData::Integer(x)])])
}

fn decoder_key(pairs: &[(PacketData, PacketData)]) -> usize {
    let dividers = [divider_packet(2), divider_packet(6)];
    let sorted_packets: Vec<_> = pairs
        .iter()
        .flat_map(|(x, y)| [x, y])
        .chain(dividers.iter())
        .sorted()
        .collect();
    (sorted_packets.iter().position(|p| *p == &dividers[0]).unwrap() + 1)*
    (sorted_packets.iter().position(|p| *p == &dividers[1]).unwrap() + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::assert_equal;

    #[test]
    fn input_is_parsed() {
        assert_eq!(parse_input(EXAMPLE_INPUT).unwrap(), example_pairs());
    }

    #[test]
    fn compare_works_as_specified() {
        use Ordering::*;
        assert_equal(
            example_pairs().iter().map(|(lhs, rhs)| std::cmp::Ord::cmp(lhs, rhs)),
            [Less, Less, Greater, Less, Greater, Less, Greater, Greater],
        );
    }

    #[test]
    fn sum_of_indices_of_correctly_ordered_pairs() {
        assert_eq!(correctly_ordered_index_sum(&example_pairs()), 13);
    }

    #[test]
    fn decoder_key_is_found() {
        assert_eq!(decoder_key(&example_pairs()), 140);
    }

    const EXAMPLE_INPUT: &str = "\
[1,1,3,1,1]
[1,1,5,1,1]

[[1],[2,3,4]]
[[1],4]

[9]
[[8,7,6]]

[[4,4],4,4]
[[4,4],4,4,4]

[7,7,7,7]
[7,7,7]

[]
[3]

[[[]]]
[[]]

[1,[2,[3,[4,[5,6,7]]]],8,9]
[1,[2,[3,[4,[5,6,0]]]],8,9]";

    fn example_pairs() -> Vec<(PacketData, PacketData)> {
        use PacketData::*;
        vec![
            (
                List(vec![
                    Integer(1),
                    Integer(1),
                    Integer(3),
                    Integer(1),
                    Integer(1),
                ]),
                List(vec![
                    Integer(1),
                    Integer(1),
                    Integer(5),
                    Integer(1),
                    Integer(1),
                ]),
            ),
            (
                List(vec![
                    List(vec![Integer(1)]),
                    List(vec![Integer(2), Integer(3), Integer(4)]),
                ]),
                List(vec![List(vec![Integer(1)]), Integer(4)]),
            ),
            (
                List(vec![Integer(9)]),
                List(vec![List(vec![Integer(8), Integer(7), Integer(6)])]),
            ),
            (
                List(vec![
                    List(vec![Integer(4), Integer(4)]),
                    Integer(4),
                    Integer(4),
                ]),
                List(vec![
                    List(vec![Integer(4), Integer(4)]),
                    Integer(4),
                    Integer(4),
                    Integer(4),
                ]),
            ),
            (
                List(vec![Integer(7), Integer(7), Integer(7), Integer(7)]),
                List(vec![Integer(7), Integer(7), Integer(7)]),
            ),
            (List(vec![]), List(vec![Integer(3)])),
            (
                List(vec![List(vec![List(vec![])])]),
                List(vec![List(vec![])]),
            ),
            (
                List(vec![
                    Integer(1),
                    List(vec![
                        Integer(2),
                        List(vec![
                            Integer(3),
                            List(vec![
                                Integer(4),
                                List(vec![Integer(5), Integer(6), Integer(7)]),
                            ]),
                        ]),
                    ]),
                    Integer(8),
                    Integer(9),
                ]),
                List(vec![
                    Integer(1),
                    List(vec![
                        Integer(2),
                        List(vec![
                            Integer(3),
                            List(vec![
                                Integer(4),
                                List(vec![Integer(5), Integer(6), Integer(0)]),
                            ]),
                        ]),
                    ]),
                    Integer(8),
                    Integer(9),
                ]),
            ),
        ]
    }
}
