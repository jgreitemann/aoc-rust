use std::{collections::HashSet, ops::RangeInclusive};

use anyhow::{Context, anyhow, bail};
use aoc_companion::prelude::*;
use itertools::Itertools as _;

pub(crate) struct Door<'input> {
    rules: Vec<Rule<'input>>,
    my_ticket: Vec<u64>,
    nearby_tickets: Vec<Vec<u64>>,
}

impl<'input> Solution<'input> for Door<'input> {
    fn parse(input: &'input str) -> Result<Self> {
        let [rules, my_ticket, nearby_tickets] =
            aoc_utils::array::from_iter_exact(input.split("\n\n"))
                .map_err(|v| anyhow!("expected three sections, got {} sections", v.len()))?;
        let rules = rules.lines().map(parse_rule).try_collect()?;
        let my_ticket = my_ticket
            .strip_prefix("your ticket:\n")
            .ok_or_else(|| anyhow!("missing introducer \"your ticket:\""))?
            .split(',')
            .map(str::parse)
            .try_collect()
            .with_context(|| anyhow!("failed to parse my ticket"))?;
        let nearby_tickets = nearby_tickets
            .strip_prefix("nearby tickets:\n")
            .ok_or_else(|| anyhow!("missing introducer \"nearby tickets:\""))?
            .lines()
            .map(|ticket| {
                ticket
                    .split(',')
                    .map(str::parse)
                    .try_collect()
                    .with_context(|| anyhow!("failed to parse nearby ticket {ticket:?}"))
            })
            .try_collect()?;
        Ok(Self {
            rules,
            my_ticket,
            nearby_tickets,
        })
    }

    fn part1(&self) -> u64 {
        self.nearby_tickets
            .iter()
            .flatten()
            .filter(|&&x| !is_valid_for_any_field(x, &self.rules))
            .sum()
    }

    fn part2(&self) -> Result<u64> {
        let mapping = find_field_mapping(self.valid_tickets(), &self.rules)?;
        Ok(self
            .rules
            .iter()
            .enumerate()
            .filter(|(_, rule)| rule.field_name.starts_with("departure"))
            .map(|(i, _)| self.my_ticket[mapping[i]])
            .product())
    }
}

impl Door<'_> {
    fn valid_tickets(&self) -> impl Iterator<Item = &[u64]> {
        self.nearby_tickets
            .iter()
            .filter(|ticket| {
                ticket
                    .iter()
                    .all(|&x| is_valid_for_any_field(x, &self.rules))
            })
            .map(Vec::as_slice)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Rule<'input> {
    field_name: &'input str,
    domain: [RangeInclusive<u64>; 2],
}

fn parse_rule<'input>(s: &'input str) -> Result<Rule<'input>> {
    let Some((field_name, ranges)) = s.split_once(':') else {
        bail!("missing colon in rule {s:?}");
    };
    Ok(Rule {
        field_name,
        domain: aoc_utils::array::try_from_iter_exact(ranges.trim_start().split("or").map(|r| {
            r.trim()
                .split_once('-')
                .ok_or_else(|| anyhow!("missing dash in range {r:?}"))
                .and_then(|(from, to)| Ok(from.parse()?..=to.parse()?))
                .with_context(|| anyhow!("failed to parse range {r:?}"))
        }))?
        .map_err(|v| anyhow!("expected exactly two ranges, got {}", v.len()))?,
    })
}

fn is_valid_for_any_field(x: u64, rules: &[Rule]) -> bool {
    rules
        .iter()
        .flat_map(|rule| rule.domain.iter())
        .any(|r| r.contains(&x))
}

fn find_field_mapping<'a>(
    tickets: impl IntoIterator<Item = &'a [u64]>,
    rules: &[Rule],
) -> Result<Vec<usize>> {
    let mut possible_indices = vec![HashSet::<_>::from_iter(0..rules.len()); rules.len()];

    for ticket in tickets {
        for (i, field) in ticket.iter().enumerate() {
            for (rule_indices, rule) in possible_indices.iter_mut().zip(rules) {
                if !rule.domain.iter().any(|r| r.contains(field)) {
                    rule_indices.remove(&i);
                }
            }
        }
    }

    let mut res = vec![None; rules.len()];
    while let Some((i, s)) = possible_indices
        .iter()
        .enumerate()
        .find(|(_, s)| s.len() == 1)
    {
        let j = *s.iter().next().unwrap();
        res[i] = Some(j);
        for p in &mut possible_indices {
            p.remove(&j);
        }
    }

    res.into_iter()
        .map(|o| o.ok_or_else(|| anyhow!("field mapping ambiguous")))
        .try_collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "\
class: 1-3 or 5-7
row: 6-11 or 33-44
seat: 13-40 or 45-50

your ticket:
7,1,14

nearby tickets:
7,3,47
40,4,50
55,2,20
38,6,12";

    const EXAMPLE_RULES: &[Rule] = &[
        Rule {
            field_name: "class",
            domain: [1..=3, 5..=7],
        },
        Rule {
            field_name: "row",
            domain: [6..=11, 33..=44],
        },
        Rule {
            field_name: "seat",
            domain: [13..=40, 45..=50],
        },
    ];

    const EXAMPLE_MY_TICKET: [u64; 3] = [7, 1, 14];

    const EXAMPLE_NEARBY_TICKETS: &[[u64; 3]] =
        &[[7, 3, 47], [40, 4, 50], [55, 2, 20], [38, 6, 12]];

    const SECOND_EXAMPLE_INPUT: &str = "\
class: 0-1 or 4-19
row: 0-5 or 8-19
seat: 0-13 or 16-19

your ticket:
11,12,13

nearby tickets:
3,9,18
15,1,5
5,14,9";

    #[test]
    fn parse_example_input() {
        let Door {
            rules,
            my_ticket,
            nearby_tickets,
        } = Door::parse(EXAMPLE_INPUT).unwrap();
        itertools::assert_equal(&rules, EXAMPLE_RULES);
        assert_eq!(my_ticket, EXAMPLE_MY_TICKET);
        itertools::assert_equal(&nearby_tickets, EXAMPLE_NEARBY_TICKETS);
    }

    #[test]
    fn find_invalid_values_in_example() {
        itertools::assert_equal(
            EXAMPLE_NEARBY_TICKETS
                .iter()
                .flatten()
                .filter(|&&x| !is_valid_for_any_field(x, EXAMPLE_RULES)),
            &[4, 55, 12],
        );
    }

    #[test]
    fn find_field_mapping_in_second_example() {
        let Door {
            rules,
            my_ticket: _,
            nearby_tickets,
        } = Door::parse(SECOND_EXAMPLE_INPUT).unwrap();
        assert_eq!(
            find_field_mapping(nearby_tickets.iter().map(Vec::as_slice), &rules).unwrap(),
            [1, 0, 2]
        );
    }
}
