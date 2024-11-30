use aoc_companion::prelude::*;

use itertools::Itertools;
use thiserror::Error;

use std::cmp::Ordering;
use std::str::FromStr;

pub(crate) struct Door {
    pairs: Vec<(PacketData, PacketData)>,
}

impl<'input> ParseInput<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseError> {
        parse_input(input).map(|pairs| Self { pairs })
    }
}

impl Part1 for Door {
    fn part1(&self) -> usize {
        correctly_ordered_index_sum(&self.pairs)
    }
}

impl Part2 for Door {
    fn part2(&self) -> usize {
        decoder_key(&self.pairs)
    }
}

#[derive(Debug, Error)]
pub(crate) enum ParseError {
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("Did not find a pair of packets")]
    NoPair,
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
                .map(PacketData::List)
        } else {
            Ok(PacketData::Integer(s.parse()?))
        }
    }
}

impl Ord for PacketData {
    fn cmp(&self, other: &Self) -> Ordering {
        use PacketData::*;
        match (self, other) {
            (Integer(x), Integer(y)) => Ord::cmp(x, y),
            (List(xs), List(ys)) => Ord::cmp(xs, ys),
            (list, Integer(y)) => Ord::cmp(list, &List(vec![Integer(*y)])),
            (Integer(x), list) => Ord::cmp(&List(vec![Integer(*x)]), list),
        }
    }
}

impl PartialOrd for PacketData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn parse_input(input: &str) -> Result<Vec<(PacketData, PacketData)>, ParseError> {
    input
        .split("\n\n")
        .map(|pair_str| pair_str.split_once("\n").ok_or(ParseError::NoPair))
        .map(|res| {
            let (lhs, rhs) = res?;
            Ok((lhs.parse()?, rhs.parse()?))
        })
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
    let index_of = |div: &PacketData| sorted_packets.iter().position(|p| *p == div).unwrap() + 1;

    index_of(&dividers[0]) * index_of(&dividers[1])
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
            example_pairs().iter().map(|(lhs, rhs)| Ord::cmp(lhs, rhs)),
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
