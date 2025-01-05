use aoc_companion::prelude::*;
use regex::Regex;

pub(crate) struct Door<'input> {
    entries: Vec<DatabaseEntry<'input>>,
}

impl<'input> Solution<'input> for Door<'input> {
    fn parse(input: &'input str) -> Self {
        let re = Regex::new(r"(\d+)-(\d+) ([a-z]): ([a-z]+)").unwrap();

        let entries = re
            .captures_iter(input)
            .map(|cap| DatabaseEntry {
                first: cap[1].parse().unwrap(),
                second: cap[2].parse().unwrap(),
                char: cap[3].as_bytes().iter().copied().next().unwrap(),
                password: cap.get(4).unwrap().as_str().as_bytes(),
            })
            .collect();

        Door { entries }
    }

    fn part1(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| DatabaseEntry::matches_count_policy(entry))
            .count()
    }

    fn part2(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| DatabaseEntry::matches_index_policy(entry))
            .count()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DatabaseEntry<'input> {
    first: usize,
    second: usize,
    char: u8,
    password: &'input [u8],
}

impl DatabaseEntry<'_> {
    fn matches_count_policy(&self) -> bool {
        (self.first..=self.second)
            .contains(&self.password.iter().filter(|&&c| c == self.char).count())
    }

    fn matches_index_policy(&self) -> bool {
        (self.password[self.first - 1] == self.char) ^ (self.password[self.second - 1] == self.char)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT: &str = "\
1-3 a: abcde
1-3 b: cdefg
2-9 c: ccccccccc";

    const EXAMPLE_ENTRIES: &[DatabaseEntry] = &[
        DatabaseEntry {
            first: 1,
            second: 3,
            char: b'a',
            password: b"abcde",
        },
        DatabaseEntry {
            first: 1,
            second: 3,
            char: b'b',
            password: b"cdefg",
        },
        DatabaseEntry {
            first: 2,
            second: 9,
            char: b'c',
            password: b"ccccccccc",
        },
    ];

    #[test]
    fn parse_example_input() {
        assert_eq!(Door::parse(EXAMPLE_INPUT).entries, EXAMPLE_ENTRIES);
    }

    #[test]
    fn example_entries_matching_count_policy() {
        assert!(EXAMPLE_ENTRIES[0].matches_count_policy());
        assert!(EXAMPLE_ENTRIES[2].matches_count_policy());

        assert!(!EXAMPLE_ENTRIES[1].matches_count_policy());
    }

    #[test]
    fn example_entries_matching_index_policy() {
        assert!(EXAMPLE_ENTRIES[0].matches_index_policy());

        assert!(!EXAMPLE_ENTRIES[1].matches_index_policy());
        assert!(!EXAMPLE_ENTRIES[2].matches_index_policy());
    }
}
