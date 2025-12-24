use anyhow::{Context, bail};
use aoc_companion::prelude::*;
use itertools::{Either, Itertools};

pub(crate) struct Door {
    rules: Vec<Rule>,
    messages: Vec<Box<[u8]>>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        let Some((rules, messages)) = input.split_once("\n\n") else {
            bail!("missing empty line separating rules from messages");
        };
        let rules: Vec<Rule> = rules
            .lines()
            .map(|line| -> Result<(usize, Rule)> {
                let Some((idx, body)) = line.split_once(':') else {
                    bail!("missing colon separating rule index from body");
                };
                let idx = idx.parse().context("failed to parse rule index")?;

                let rule = if let Some(lit_str) = body.trim().strip_prefix('"') {
                    let Some(lit_str) = lit_str.strip_suffix('"') else {
                        bail!("missing closing quotation mark in string literal rule");
                    };
                    let &[byte] = lit_str.as_bytes() else {
                        bail!("string literal rule is more than one ASCII character");
                    };
                    Rule::Literal(byte)
                } else {
                    body.split('|')
                        .map(|alt| {
                            alt.split_whitespace()
                                .map(|ref_str| {
                                    ref_str
                                        .parse()
                                        .context("failed to parse rule reference index")
                                        .map(RuleRef)
                                })
                                .try_collect()
                                .map(All)
                        })
                        .try_collect()
                        .map(Rule::Any)?
                };

                Ok((idx, rule))
            })
            .try_fold(Vec::new(), |mut rules, res| -> Result<Vec<Rule>> {
                let (idx, rule) = res?;
                if rules.len() <= idx {
                    rules.resize_with(idx + 1, || Rule::Any(vec![]));
                }
                rules[idx] = rule;
                Ok(rules)
            })?;

        let messages = messages
            .lines()
            .map(|line| line.as_bytes().to_vec().into_boxed_slice())
            .collect();

        Ok(Self { rules, messages })
    }

    fn part1(&self) -> usize {
        self.messages
            .iter()
            .filter(matches(RuleRef(0), &self.rules))
            .count()
    }
}

fn munch<'c>(
    RuleRef(rule_idx): RuleRef,
    rules: &[Rule],
    candidate: &'c [u8],
) -> impl Iterator<Item = &'c [u8]> {
    let rule = &rules[rule_idx];
    match rule {
        Rule::Literal(b) => Either::Left(candidate.strip_prefix(&[*b]).into_iter()),
        Rule::Any(alternatives) => {
            Either::Right(alternatives.iter().flat_map(move |All(rule_seq)| {
                rule_seq
                    .iter()
                    .copied()
                    .fold(vec![candidate], |rests, rule_ref| {
                        rests
                            .into_iter()
                            .flat_map(|rest| munch(rule_ref, rules, rest))
                            .collect()
                    })
            }))
        }
    }
}

fn matches<C: AsRef<[u8]>>(rule_ref: RuleRef, rules: &[Rule]) -> impl Fn(&C) -> bool {
    move |candidate| munch(rule_ref, rules, candidate.as_ref()).contains(b"".as_slice())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct RuleRef(usize);

#[derive(Clone, Debug, PartialEq, Eq)]
struct All(Vec<RuleRef>);

#[derive(Clone, Debug, PartialEq, Eq)]
enum Rule {
    Literal(u8),
    Any(Vec<All>),
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = r#"0: 4 1 5
1: 2 3 | 3 2
2: 4 4 | 5 5
3: 4 5 | 5 4
4: "a"
5: "b"

ababbb
bababa
abbbab
aaabbb
aaaabbb"#;

    fn example_rules() -> Vec<Rule> {
        vec![
            Rule::Any(vec![All(vec![RuleRef(4), RuleRef(1), RuleRef(5)])]),
            Rule::Any(vec![
                All(vec![RuleRef(2), RuleRef(3)]),
                All(vec![RuleRef(3), RuleRef(2)]),
            ]),
            Rule::Any(vec![
                All(vec![RuleRef(4), RuleRef(4)]),
                All(vec![RuleRef(5), RuleRef(5)]),
            ]),
            Rule::Any(vec![
                All(vec![RuleRef(4), RuleRef(5)]),
                All(vec![RuleRef(5), RuleRef(4)]),
            ]),
            Rule::Literal(b'a'),
            Rule::Literal(b'b'),
        ]
    }

    const EXAMPLE_MESSAGES: &[&[u8]] = &[b"ababbb", b"bababa", b"abbbab", b"aaabbb", b"aaaabbb"];

    #[test]
    fn parse_example_input() {
        let Door { rules, messages } = Door::parse(EXAMPLE_INPUT).unwrap();
        itertools::assert_equal(rules, example_rules());
        itertools::assert_equal(
            messages.iter().map(|m| m.as_ref()),
            EXAMPLE_MESSAGES.iter().copied(),
        );
    }

    #[test]
    fn munch_some_rules() {
        let rules = example_rules();
        itertools::assert_equal(munch(RuleRef(4), &rules, b"ababbb"), [b"babbb"]);
        itertools::assert_equal(munch(RuleRef(3), &rules, b"babbb"), [b"bbb"]);
        itertools::assert_equal(
            munch(RuleRef(2), &rules, b"babbb"),
            std::iter::empty::<&[u8]>(),
        );
        itertools::assert_equal(munch(RuleRef(2), &rules, b"bbb"), [b"b"]);
        itertools::assert_equal(munch(RuleRef(1), &rules, b"babbb"), [b"b"]);
        itertools::assert_equal(munch(RuleRef(0), &rules, b"ababbb"), [b""]);
    }

    #[test]
    fn matching_example_messages() {
        itertools::assert_equal(
            EXAMPLE_MESSAGES
                .iter()
                .map(matches(RuleRef(0), &example_rules())),
            [true, false, true, false, false],
        );
    }
}
