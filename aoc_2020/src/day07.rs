use anyhow::bail;
use aoc_companion::prelude::*;

use aoc_utils::cache::cached;
use itertools::Itertools;
use regex::Regex;
use std::collections::HashMap;

pub(crate) struct Door<'input> {
    regulations: HashMap<Color<'input>, Vec<Requirement<'input>>>,
}

impl<'input> Solution<'input> for Door<'input> {
    fn parse(input: &'input str) -> Result<Self> {
        let outer_re =
            Regex::new(r"([a-z]+ [a-z]+) bags contain ((?:\d+ [a-z]+ [a-z]+ bags?(?:, )?)+)\.")
                .unwrap();

        let regulations = outer_re
            .captures_iter(input)
            .map(|cap| cap.extract())
            .map(|(_, [color, requirements])| {
                Ok::<_, anyhow::Error>((
                    Color(color),
                    requirements
                        .split(", ")
                        .map(Requirement::parse)
                        .try_collect()?,
                ))
            })
            .try_collect()?;

        Ok(Door { regulations })
    }

    fn part1(&self) -> usize {
        number_of_bags_containing_a_shiny_gold_bag(&self.regulations)
    }

    fn part2(&self) -> usize {
        number_of_bags_in_total(&self.regulations, SHINY_GOLD) - 1
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
struct Color<'input>(&'input str);

const SHINY_GOLD: Color = Color("shiny gold");

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Requirement<'input> {
    quantity: usize,
    color: Color<'input>,
}

impl<'input> Requirement<'input> {
    fn parse(req: &'input str) -> Result<Requirement<'input>> {
        let inner_re = Regex::new(r"^(\d+) ([a-z]+ [a-z]+) bags?$").unwrap();
        let Some((_, [quantity, color])) = inner_re.captures(req).map(|cap| cap.extract()) else {
            bail!("requirement regex didn't match");
        };

        Ok(Requirement {
            quantity: quantity.parse()?,
            color: Color(color),
        })
    }
}

fn contains_shiny_gold_bag<'input>(
    regulations: &HashMap<Color<'input>, impl AsRef<[Requirement<'input>]>>,
    bag: Color<'input>,
) -> bool {
    regulations.get(&bag).is_some_and(|this_bags_requirements| {
        this_bags_requirements
            .as_ref()
            .iter()
            .any(|r| r.color == SHINY_GOLD)
            || this_bags_requirements
                .as_ref()
                .iter()
                .any(|r| contains_shiny_gold_bag(regulations, r.color))
    })
}

fn number_of_bags_containing_a_shiny_gold_bag<'input>(
    regulations: &HashMap<Color<'input>, impl AsRef<[Requirement<'input>]>>,
) -> usize {
    regulations
        .keys()
        .filter(|&&c| contains_shiny_gold_bag(regulations, c))
        .count()
}

fn number_of_bags_in_total<'input>(
    regulations: &HashMap<Color<'input>, impl AsRef<[Requirement<'input>]>>,
    bag: Color<'input>,
) -> usize {
    let mut cached_func = cached(move |bag, recurse| {
        regulations
            .get(&bag)
            .iter()
            .map(move |requirements| {
                requirements
                    .as_ref()
                    .iter()
                    .map(|Requirement { quantity, color }| *quantity * recurse(*color))
                    .sum::<usize>()
            })
            .sum::<usize>()
            + 1
    });

    cached_func(bag)
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT: &str = "\
light red bags contain 1 bright white bag, 2 muted yellow bags.
dark orange bags contain 3 bright white bags, 4 muted yellow bags.
bright white bags contain 1 shiny gold bag.
muted yellow bags contain 2 shiny gold bags, 9 faded blue bags.
shiny gold bags contain 1 dark olive bag, 2 vibrant plum bags.
dark olive bags contain 3 faded blue bags, 4 dotted black bags.
vibrant plum bags contain 5 faded blue bags, 6 dotted black bags.
faded blue bags contain no other bags.
dotted black bags contain no other bags.";

    const EXAMPLE_REGULATIONS: [(Color, &[Requirement]); 7] = [
        (
            Color("light red"),
            &[
                Requirement {
                    quantity: 1,
                    color: Color("bright white"),
                },
                Requirement {
                    quantity: 2,
                    color: Color("muted yellow"),
                },
            ],
        ),
        (
            Color("dark orange"),
            &[
                Requirement {
                    quantity: 3,
                    color: Color("bright white"),
                },
                Requirement {
                    quantity: 4,
                    color: Color("muted yellow"),
                },
            ],
        ),
        (
            Color("bright white"),
            &[Requirement {
                quantity: 1,
                color: Color("shiny gold"),
            }],
        ),
        (
            Color("muted yellow"),
            &[
                Requirement {
                    quantity: 2,
                    color: Color("shiny gold"),
                },
                Requirement {
                    quantity: 9,
                    color: Color("faded blue"),
                },
            ],
        ),
        (
            Color("shiny gold"),
            &[
                Requirement {
                    quantity: 1,
                    color: Color("dark olive"),
                },
                Requirement {
                    quantity: 2,
                    color: Color("vibrant plum"),
                },
            ],
        ),
        (
            Color("dark olive"),
            &[
                Requirement {
                    quantity: 3,
                    color: Color("faded blue"),
                },
                Requirement {
                    quantity: 4,
                    color: Color("dotted black"),
                },
            ],
        ),
        (
            Color("vibrant plum"),
            &[
                Requirement {
                    quantity: 5,
                    color: Color("faded blue"),
                },
                Requirement {
                    quantity: 6,
                    color: Color("dotted black"),
                },
            ],
        ),
    ];

    #[test]
    fn parse_example_input() {
        let Door { regulations } = Door::parse(EXAMPLE_INPUT).unwrap();
        assert_eq!(
            regulations,
            HashMap::from(EXAMPLE_REGULATIONS.map(|(color, req)| (color, req.to_vec())))
        );
    }

    #[test]
    fn bright_white_bag_contains_shiny_gold_bag() {
        assert!(contains_shiny_gold_bag(
            &HashMap::from(EXAMPLE_REGULATIONS),
            Color("bright white")
        ));
    }

    #[test]
    fn dark_orange_bag_contains_shiny_gold_bag() {
        assert!(contains_shiny_gold_bag(
            &HashMap::from(EXAMPLE_REGULATIONS),
            Color("dark orange")
        ));
    }

    #[test]
    fn faded_blue_bag_does_not_contain_shiny_gold_bag() {
        assert!(!contains_shiny_gold_bag(
            &HashMap::from(EXAMPLE_REGULATIONS),
            Color("faded blue")
        ));
    }

    #[test]
    fn number_of_bags_containing_a_shiny_gold_bag() {
        assert_eq!(
            super::number_of_bags_containing_a_shiny_gold_bag(&HashMap::from(EXAMPLE_REGULATIONS)),
            4
        );
    }

    #[test]
    fn number_of_bags_in_total() {
        assert_eq!(
            super::number_of_bags_in_total(&HashMap::from(EXAMPLE_REGULATIONS), SHINY_GOLD),
            33
        );
    }
}
