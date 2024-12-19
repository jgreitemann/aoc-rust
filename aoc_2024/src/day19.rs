use anyhow::bail;
use aoc_companion::prelude::*;
use regex::Regex;

pub(crate) struct Door<'input> {
    towels: Vec<&'input str>,
    patterns: Vec<&'input str>,
}

impl<'input> Solution<'input> for Door<'input> {
    fn parse(input: &'input str) -> Result<Self> {
        let Some((towels_str, patterns_str)) = input.split_once("\n\n") else {
            bail!("Missing empty line separating towels from patterns");
        };
        let towels = towels_str.split(", ").collect();
        let patterns = patterns_str.lines().collect();
        Ok(Door { towels, patterns })
    }

    fn part1(&self) -> usize {
        self.matching_patterns().count()
    }

    fn part2(&self) -> usize {
        self.number_of_matches()
    }
}

impl<'input> Door<'input> {
    fn matching_patterns(&'input self) -> impl Iterator<Item = &'input str> {
        let re = Regex::new(&format!("^(?:{})*$", self.towels.join("|"))).unwrap();
        self.patterns
            .iter()
            .copied()
            .filter(move |p| re.is_match(p))
    }

    fn number_of_matches(&self) -> usize {
        self.patterns
            .iter()
            .inspect(|x| eprintln!("{x}"))
            .map(|pattern| number_of_matches_for_pattern(pattern, &self.towels))
            .sum()
    }
}

fn number_of_matches_for_pattern(pattern: &str, towels: &[&str]) -> usize {
    towels
        .iter()
        .map(|towel| match pattern.strip_prefix(towel) {
            None => 0,
            Some("") => 1,
            Some(rest) => number_of_matches_for_pattern(rest, towels),
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use itertools::assert_equal;
    use proptest::proptest;

    use super::*;

    const EXAMPLE_INPUT: &str = "r, wr, b, g, bwu, rb, gb, br

brwrr
bggr
gbbr
rrbgbr
ubwu
bwurrg
brgr
bbrgwb";

    const EXAMPLE_TOWELS: &[&str] = &["r", "wr", "b", "g", "bwu", "rb", "gb", "br"];
    const EXAMPLE_PATTERNS: &[&str] = &[
        "brwrr", "bggr", "gbbr", "rrbgbr", "ubwu", "bwurrg", "brgr", "bbrgwb",
    ];
    const MATCHING_EXAMPLE_PATTERNS: &[&str] =
        &["brwrr", "bggr", "gbbr", "rrbgbr", "bwurrg", "brgr"];

    #[test]
    fn parse_example_input() {
        let door = Door::parse(EXAMPLE_INPUT).unwrap();
        assert_eq!(door.towels, EXAMPLE_TOWELS);
        assert_eq!(door.patterns, EXAMPLE_PATTERNS);
    }

    #[test]
    fn matching_example_patterns() {
        let door = Door {
            towels: EXAMPLE_TOWELS.to_vec(),
            patterns: EXAMPLE_PATTERNS.to_vec(),
        };
        assert_equal(
            door.matching_patterns(),
            MATCHING_EXAMPLE_PATTERNS.iter().copied(),
        );
    }

    proptest! {

        #[test]
        fn pattern_matches_are_same_for_both_methods(pattern in "[rwbgux]+") {
            let door = Door {
                towels: EXAMPLE_TOWELS.to_vec(),
                patterns: vec![&pattern],
            };
            assert_eq!(door.number_of_matches() > 0, door.matching_patterns().count() > 0);
        }

    }

    #[test]
    fn number_of_example_matches() {
        let door = Door {
            towels: EXAMPLE_TOWELS.to_vec(),
            patterns: EXAMPLE_PATTERNS.to_vec(),
        };
        assert_eq!(door.number_of_matches(), 16);
    }
}
