use aoc_companion::prelude::*;

use itertools::Itertools;
use thiserror::Error;

pub(crate) struct Door<'input> {
    signal: &'input str,
}

impl<'input> Solution<'input> for Door<'input> {
    fn parse(input: &'input str) -> Self {
        Door { signal: input }
    }

    fn part1(&self) -> Result<usize, Error> {
        disjoint_subseq_index(self.signal, 4).ok_or(Error::NoPacket)
    }

    fn part2(&self) -> Result<usize, Error> {
        disjoint_subseq_index(self.signal, 14).ok_or(Error::NoMessage)
    }
}

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("No start of packet marker found")]
    NoPacket,
    #[error("No start of message marker found")]
    NoMessage,
}

fn disjoint_subseq_index(signal: &str, n: usize) -> Option<usize> {
    signal
        .as_bytes()
        .windows(n)
        .position(|window| window.iter().all_unique())
        .map(|i| i + n)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_1: &str = "mjqjpqmgbljsphdztnvjfqwrcgsmlb";
    const EXAMPLE_2: &str = "bvwbjplbgvbhsrlpgdmjqwftvncz";
    const EXAMPLE_3: &str = "nppdvjthqldpwncqszvftbrmjlhg";
    const EXAMPLE_4: &str = "nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg";
    const EXAMPLE_5: &str = "zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw";

    #[test]
    fn find_start_of_paket_marker() {
        assert_eq!(disjoint_subseq_index(EXAMPLE_1, 4), Some(7));
        assert_eq!(disjoint_subseq_index(EXAMPLE_2, 4), Some(5));
        assert_eq!(disjoint_subseq_index(EXAMPLE_3, 4), Some(6));
        assert_eq!(disjoint_subseq_index(EXAMPLE_4, 4), Some(10));
        assert_eq!(disjoint_subseq_index(EXAMPLE_5, 4), Some(11));
    }

    #[test]
    fn find_start_of_message_marker() {
        assert_eq!(disjoint_subseq_index(EXAMPLE_1, 14), Some(19));
        assert_eq!(disjoint_subseq_index(EXAMPLE_2, 14), Some(23));
        assert_eq!(disjoint_subseq_index(EXAMPLE_3, 14), Some(23));
        assert_eq!(disjoint_subseq_index(EXAMPLE_4, 14), Some(29));
        assert_eq!(disjoint_subseq_index(EXAMPLE_5, 14), Some(26));
    }
}
