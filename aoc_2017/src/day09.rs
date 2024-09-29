use aoc_companion::prelude::*;

pub struct Door<'input> {
    stream: &'input str,
}

impl<'input> ParseInput<'input> for Door<'input> {
    type Error = std::convert::Infallible;

    fn parse(input: &'input str) -> Result<Self, Self::Error> {
        Ok(Self { stream: input })
    }
}

impl Part1 for Door<'_> {
    type Output = usize;
    type Error = std::convert::Infallible;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        Ok(stream_group_scores(self.stream).sum())
    }
}

impl Part2 for Door<'_> {
    type Output = usize;
    type Error = std::convert::Infallible;

    fn part2(&self) -> Result<Self::Output, Self::Error> {
        Ok(self.stream.chars().ignore_bangs().count_garbage())
    }
}

trait StreamIterator: Iterator<Item = char> + Sized {
    fn ignore_bangs(self) -> IgnoreBangs<Self> {
        IgnoreBangs { iter: self }
    }

    fn skip_garbage(self) -> impl Iterator<Item = char> {
        IdentifyGarbage { iter: self }.filter_map(|elem| match elem {
            StreamElement::ValidChar(c) => Some(c),
            StreamElement::GarbageRun { .. } => None,
        })
    }

    fn count_garbage(self) -> usize {
        IdentifyGarbage { iter: self }
            .filter_map(|elem| match elem {
                StreamElement::ValidChar(_) => None,
                StreamElement::GarbageRun { length } => Some(length),
            })
            .sum()
    }

    fn group_scores(self) -> GroupScores<Self> {
        GroupScores {
            iter: self,
            nesting_level: 0,
        }
    }
}

impl<I> StreamIterator for I where I: Iterator<Item = char> {}

fn stream_group_scores(stream: &str) -> impl Iterator<Item = usize> + '_ {
    stream.chars().ignore_bangs().skip_garbage().group_scores()
}

struct IgnoreBangs<I: Iterator<Item = char>> {
    iter: I,
}

impl<I> Iterator for IgnoreBangs<I>
where
    I: Iterator<Item = char>,
{
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let mut ignore_next = false;
        for c in self.iter.by_ref() {
            if std::mem::replace(&mut ignore_next, false) {
                continue;
            }
            if c == '!' {
                ignore_next = true;
                continue;
            }
            return Some(c);
        }
        None
    }
}

enum StreamElement {
    ValidChar(char),
    GarbageRun { length: usize },
}

struct IdentifyGarbage<I: Iterator<Item = char>> {
    iter: I,
}

impl<I> Iterator for IdentifyGarbage<I>
where
    I: Iterator<Item = char>,
{
    type Item = StreamElement;

    fn next(&mut self) -> Option<Self::Item> {
        let mut garbage_run = false;
        let mut length = 0;

        for c in self.iter.by_ref() {
            if garbage_run {
                match c {
                    '>' => return Some(StreamElement::GarbageRun { length }),
                    _ => length += 1,
                }
            } else {
                match c {
                    '<' => garbage_run = true,
                    c => return Some(StreamElement::ValidChar(c)),
                }
            }
        }
        None
    }
}

struct GroupScores<I: Iterator<Item = char>> {
    iter: I,
    nesting_level: usize,
}

impl<I> Iterator for GroupScores<I>
where
    I: Iterator<Item = char>,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        for c in self.iter.by_ref() {
            match c {
                '{' => {
                    self.nesting_level += 1;
                    return Some(self.nesting_level);
                }
                '}' => self.nesting_level -= 1,
                _ => {}
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("<{}>", "<{}>")]
    #[case("<{!>}>", "<{}>")]
    #[case("<!!>", "<>")]
    #[case("<!!!>>", "<>")]
    #[case("{{<!>},{<!>},{<!>},{<a>}}", "{{<},{<},{<},{<a>}}")]
    #[case("{{<!!>},{<!!>},{<!!>},{<!!>}}", "{{<>},{<>},{<>},{<>}}")]
    #[case("trailing!", "trailing")]
    fn bangs_and_following_character_are_skipped(#[case] before: &str, #[case] after: &str) {
        assert_eq!(before.chars().ignore_bangs().collect::<String>(), after);
    }

    #[rstest]
    #[case("{}", "{}")]
    #[case("{<>}", "{}")]
    #[case("{<random characters>}", "{}")]
    #[case("{<<<<>}", "{}")]
    #[case("{<a>,<a>,<a>,<a>}", "{,,,}")]
    #[case("{{<},{<},{<},{<a>}}", "{{}}")]
    fn garbage_is_skipped(#[case] before: &str, #[case] after: &str) {
        assert_eq!(before.chars().skip_garbage().collect::<String>(), after);
    }

    #[rstest]
    #[case("no groups", &[])]
    #[case("{}", &[1])]
    #[case("one {group}", &[1])]
    #[case("{{{}}}", &[1, 2, 3])]
    #[case("{}{}", &[1, 1])]
    #[case("{{}{}}", &[1, 2, 2])]
    #[case("{{{},{},{{}}}}", &[1, 2, 3, 3, 3, 4])]
    fn group_scores_without_garbage_or_bangs(#[case] before: &str, #[case] after: &[usize]) {
        assert_eq!(
            before.chars().group_scores().collect::<Vec<_>>().as_slice(),
            after
        );
    }

    #[rstest]
    #[case("{<a>,<a>,<a>,<a>}", 1)]
    #[case("{{<ab>},{<ab>},{<ab>},{<ab>}}", 9)]
    #[case("{{<!!>},{<!!>},{<!!>},{<!!>}}", 9)]
    #[case("{{<a!>},{<a!>},{<a!>},{<ab>}}", 3)]
    fn stream_group_score_sums(#[case] stream: &str, #[case] total_score: usize) {
        assert_eq!(stream_group_scores(stream).sum::<usize>(), total_score);
    }

    #[rstest]
    #[case("<>", 0)]
    #[case("<random characters>", 17)]
    #[case("<<<<>", 3)]
    #[case("<{!>}>", 2)]
    #[case("<!!>", 0)]
    #[case("<!!!>>", 0)]
    #[case(r#"<{o"i!a,<{i<a>"#, 10)]
    fn count_garbabe(#[case] stream: &str, #[case] total_garbage: usize) {
        assert_eq!(stream.chars().ignore_bangs().count_garbage(), total_garbage);
    }
}
