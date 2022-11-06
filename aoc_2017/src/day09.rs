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
        Ok(stream_group_scores(&self.stream).sum())
    }
}

trait StreamIterator: Iterator<Item = char> + Sized {
    fn ignore_bangs(self) -> IgnoreBangs<Self> {
        IgnoreBangs { iter: self }
    }

    fn skip_garbage(self) -> SkipGarbage<Self> {
        SkipGarbage { iter: self }
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
        while let Some(c) = self.iter.next() {
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

struct SkipGarbage<I: Iterator<Item = char>> {
    iter: I,
}

impl<I> Iterator for SkipGarbage<I>
where
    I: Iterator<Item = char>,
{
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let mut garbage_run = false;
        while let Some(c) = self.iter.next() {
            if garbage_run {
                match c {
                    '>' => garbage_run = false,
                    _ => continue,
                }
            } else {
                match c {
                    '<' => garbage_run = true,
                    _ => return Some(c),
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
        while let Some(c) = self.iter.next() {
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
}
