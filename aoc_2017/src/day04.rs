use aoc_companion::prelude::*;
use std::{collections::HashSet, convert::Infallible};

pub struct Door {
    passphrases: Vec<Vec<String>>,
}

impl ParseInput<'_> for Door {
    type Error = Infallible;

    fn parse(input: &str) -> Result<Self, Self::Error> {
        let passphrases = input
            .lines()
            .map(|line| line.split_whitespace().map(|s| s.to_string()).collect())
            .collect();
        Ok(Self { passphrases })
    }
}

impl Part1 for Door {
    type Output = usize;
    type Error = Infallible;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        Ok(count_valid_passphrases(
            &self.passphrases,
            passphrase_has_no_duplicates,
        ))
    }
}

impl Part2 for Door {
    type Output = usize;
    type Error = Infallible;

    fn part2(&self) -> Result<Self::Output, Self::Error> {
        Ok(count_valid_passphrases(
            &self.passphrases,
            passphrase_has_no_anagrams,
        ))
    }
}

fn count_valid_passphrases<R, S>(phrases: &[R], pred: fn(&[S]) -> bool) -> usize
where
    R: AsRef<[S]>,
    S: AsRef<str>,
{
    phrases
        .iter()
        .filter(|phrase| pred(phrase.as_ref()))
        .count()
}

fn passphrase_has_no_duplicates(words: &[impl AsRef<str>]) -> bool {
    let unique: HashSet<&str> = words.iter().map(AsRef::as_ref).collect();
    words.len() == unique.len()
}

fn passphrase_has_no_anagrams(words: &[impl AsRef<str>]) -> bool {
    use itertools::Itertools;
    let sorted_words: Vec<String> = words
        .iter()
        .map(|word| word.as_ref().chars().sorted().collect())
        .collect();
    passphrase_has_no_duplicates(&sorted_words)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicates_in_passphrases_are_detected() {
        assert!(
            passphrase_has_no_duplicates(&["aa", "bb", "cc", "dd", "ee"])
        );
        assert!(
            !passphrase_has_no_duplicates(&["aa", "bb", "cc", "dd", "aa"])
        );
        assert!(
            passphrase_has_no_duplicates(&["aa", "bb", "cc", "dd", "aaa"])
        );
    }

    #[test]
    fn anagrams_in_passphrases_are_detected() {
        assert!(passphrase_has_no_anagrams(&["abcde", "fghij"]));
        assert!(
            !passphrase_has_no_anagrams(&["abcde", "xyz", "ecdab"])
        );
        assert!(
            passphrase_has_no_anagrams(&["a", "ab", "abc", "abd", "abf", "abj"])
        );
        assert!(
            passphrase_has_no_anagrams(&["iiii", "oiii", "ooii", "oooi", "oooo"])
        );
        assert!(
            !passphrase_has_no_anagrams(&["oiii", "ioii", "iioi", "iiio"])
        );
    }
}
