use std::collections::VecDeque;

use anyhow::Context as _;
use anyhow::bail;
use aoc_companion::prelude::*;
use itertools::Itertools as _;

pub(crate) struct Door {
    decks: [Vec<u64>; 2],
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        let Some((player1, player2)) = input.split_once("\n\n") else {
            bail!("missing empty line between player decks");
        };
        let Some(player1) = player1.strip_prefix("Player 1:\n") else {
            bail!("missing introducer of player 1's deck");
        };
        let Some(player2) = player2.strip_prefix("Player 2:\n") else {
            bail!("missing introducer of player 2's deck");
        };
        aoc_utils::array::try_map([player1, player2], |deck| {
            deck.lines()
                .map(str::parse)
                .try_collect()
                .with_context(|| "failed to parse card value")
        })
        .map(|decks| Door { decks })
    }

    fn part1(&self) -> u64 {
        score(&play_combat(
            self.decks.each_ref().map(|deck| deck.clone().into()),
        ))
    }
}

fn play_combat(mut decks: [VecDeque<u64>; 2]) -> Vec<u64> {
    while let Ok(top_cards) =
        aoc_utils::array::try_map(decks.each_mut(), |deck| deck.front().copied().ok_or(()))
    {
        for deck in &mut decks {
            deck.pop_front();
        }
        match (top_cards, &mut decks) {
            ([high, low], [winner, _]) | ([low, high], [_, winner]) if high > low => {
                winner.extend([high, low]);
            }
            _ => unreachable!("same card value must not occur twice"),
        }
    }

    let winner = decks.into_iter().max().unwrap();
    winner.into()
}

fn score(deck: &[u64]) -> u64 {
    deck.iter()
        .rev()
        .enumerate()
        .map(|(i, card)| (i as u64 + 1) * card)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "\
Player 1:
9
2
6
3
1

Player 2:
5
8
4
7
10";

    const EXAMPLE_DECKS: [[u64; 5]; 2] = [[9, 2, 6, 3, 1], [5, 8, 4, 7, 10]];
    const WINNING_DECK: [u64; 10] = [3, 2, 10, 6, 8, 5, 9, 4, 7, 1];

    #[test]
    fn parse_example_input() {
        let Door { decks } = Door::parse(EXAMPLE_INPUT).unwrap();
        assert_eq!(decks, EXAMPLE_DECKS);
    }

    #[test]
    fn winning_deck_in_example() {
        assert_eq!(play_combat(EXAMPLE_DECKS.map(VecDeque::from)), WINNING_DECK);
    }

    #[test]
    fn winning_score() {
        assert_eq!(score(&WINNING_DECK), 306);
    }
}
