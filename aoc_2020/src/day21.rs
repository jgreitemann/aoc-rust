use std::{
    borrow::Borrow,
    collections::{BTreeMap, BTreeSet, HashMap},
};

use anyhow::bail;
use aoc_companion::prelude::*;
use aoc_utils::iter::IterUtils;
use itertools::Itertools as _;

pub(crate) struct Door<'a> {
    foods: HashMap<BTreeSet<Ingredient<'a>>, BTreeSet<Allergen<'a>>>,
}

impl<'input> Solution<'input> for Door<'input> {
    fn parse(input: &'input str) -> Result<Self> {
        input
            .lines()
            .map(|line| {
                let Some((ingredients, allergens)) = line.split_once(" (contains ") else {
                    bail!("missing allergens in food spec");
                };
                let Some(allergens) = allergens.strip_suffix(')') else {
                    bail!("missing closing parenthesis after allergen list");
                };
                Ok((
                    ingredients
                        .split_ascii_whitespace()
                        .map(Ingredient)
                        .collect(),
                    allergens.split(", ").map(Allergen).collect(),
                ))
            })
            .try_collect()
            .map(|foods| Door { foods })
    }

    fn part1(&self) -> usize {
        safe_ingredients(&self.foods).count()
    }

    fn part2(&self) -> String {
        let inv_mapping: BTreeMap<_, _> = infer_mapping(&self.foods)
            .into_iter()
            .map(|(a, i)| (i, a))
            .collect();

        inv_mapping.values().join(",")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Ingredient<'a>(&'a str);

impl std::fmt::Display for Ingredient<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Allergen<'a>(&'a str);

fn intersect<T: Copy + Ord>(other: &mut BTreeSet<T>, new: BTreeSet<T>) {
    *other = other.intersection(&new).copied().collect();
}

fn infer_mapping<'a, Is, As>(
    foods: impl IntoIterator<Item = (Is, As)>,
) -> HashMap<Ingredient<'a>, Allergen<'a>>
where
    Is: Borrow<BTreeSet<Ingredient<'a>>> + Ord,
    As: Borrow<BTreeSet<Allergen<'a>>> + Ord,
{
    let mut relations = foods
        .into_iter()
        .map(|(i, a)| (a.borrow().clone(), i.borrow().clone()))
        .btree_merge(intersect);

    let mut solution = HashMap::new();

    while !relations.is_empty() {
        dbg!(relations.len());
        // Generate non-empty intersections
        let mut intersections = relations
            .iter()
            .tuple_combinations()
            .filter_map(|((lhs_a, lhs_i), (rhs_a, rhs_i))| {
                let int_a: BTreeSet<_> = lhs_a.intersection(rhs_a).copied().collect();
                if int_a.is_empty() {
                    return None;
                }
                let int_i = lhs_i.intersection(rhs_i).copied().collect();
                Some((int_a, int_i))
            })
            .btree_merge(intersect);
        relations.append(&mut intersections);

        // Identify and eliminate unique relations
        while let Some((allergen, ingredient)) =
            relations.iter().find_map(|(allergens, ingredients)| {
                let allergen = allergens.iter().exactly_one().ok()?;
                let ingredient = ingredients
                    .iter()
                    .filter(|&i| !solution.contains_key(i))
                    .exactly_one()
                    .ok()?;
                Some((*allergen, *ingredient))
            })
        {
            solution.insert(ingredient, allergen);

            relations = relations
                .into_iter()
                .map(|(mut allergens, mut ingredients)| {
                    allergens.remove(&allergen);
                    ingredients.remove(&ingredient);
                    (allergens, ingredients)
                })
                .filter(|(allergens, _)| !allergens.is_empty())
                .btree_merge(intersect);
        }
    }

    solution
}

fn safe_ingredients<'a>(
    foods: &HashMap<BTreeSet<Ingredient<'a>>, BTreeSet<Allergen<'a>>>,
) -> impl Iterator<Item = Ingredient<'a>> {
    let mapping = infer_mapping(foods);
    foods
        .keys()
        .flatten()
        .copied()
        .filter(move |ingredient| !mapping.contains_key(ingredient))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    const EXAMPLE_INPUT: &str = "\
mxmxvkd kfcds sqjhc nhms (contains dairy, fish)
trh fvjkl sbzzf mxmxvkd (contains dairy)
sqjhc fvjkl (contains soy)
sqjhc mxmxvkd sbzzf (contains fish)";

    const EXAMPLE_FOODS: [(&[Ingredient], &[Allergen]); 4] = [
        (
            &[
                Ingredient("mxmxvkd"),
                Ingredient("kfcds"),
                Ingredient("sqjhc"),
                Ingredient("nhms"),
            ],
            &[Allergen("dairy"), Allergen("fish")],
        ),
        (
            &[
                Ingredient("trh"),
                Ingredient("fvjkl"),
                Ingredient("sbzzf"),
                Ingredient("mxmxvkd"),
            ],
            &[Allergen("dairy")],
        ),
        (
            &[Ingredient("sqjhc"), Ingredient("fvjkl")],
            &[Allergen("soy")],
        ),
        (
            &[
                Ingredient("sqjhc"),
                Ingredient("mxmxvkd"),
                Ingredient("sbzzf"),
            ],
            &[Allergen("fish")],
        ),
    ];

    const EXAMPLE_MAPPING: [(Ingredient, Allergen); 3] = [
        (Ingredient("mxmxvkd"), Allergen("dairy")),
        (Ingredient("sqjhc"), Allergen("fish")),
        (Ingredient("fvjkl"), Allergen("soy")),
    ];

    const CUSTOM_FOODS: [(&[Ingredient], &[Allergen]); 6] = [
        (
            &[
                Ingredient("A"),
                Ingredient("B"),
                Ingredient("C"),
                Ingredient("D"),
                Ingredient("E"),
                Ingredient("F"),
                Ingredient("G"),
            ],
            &[Allergen("fish"), Allergen("nuts"), Allergen("eggs")],
        ),
        (
            &[
                Ingredient("A"),
                Ingredient("B"),
                Ingredient("D"),
                Ingredient("E"),
            ],
            &[Allergen("fish"), Allergen("nuts")],
        ),
        (
            &[
                Ingredient("A"),
                Ingredient("B"),
                Ingredient("C"),
                Ingredient("F"),
            ],
            &[Allergen("fish"), Allergen("eggs")],
        ),
        (
            &[
                Ingredient("A"),
                Ingredient("C"),
                Ingredient("D"),
                Ingredient("G"),
            ],
            &[Allergen("fish"), Allergen("plutonium")],
        ),
        (
            &[
                Ingredient("B"),
                Ingredient("C"),
                Ingredient("D"),
                Ingredient("F"),
                Ingredient("G"),
            ],
            &[Allergen("nuts"), Allergen("plutonium")],
        ),
        (
            &[Ingredient("C"), Ingredient("D"), Ingredient("E")],
            &[Allergen("eggs"), Allergen("plutonium")],
        ),
    ];

    const CUSTOM_MAPPING: [(Ingredient, Allergen); 4] = [
        (Ingredient("A"), Allergen("fish")),
        (Ingredient("B"), Allergen("nuts")),
        (Ingredient("C"), Allergen("eggs")),
        (Ingredient("D"), Allergen("plutonium")),
    ];

    #[test]
    fn parse_example_input() {
        let Door { foods } = Door::parse(EXAMPLE_INPUT).unwrap();
        assert_eq!(
            foods,
            HashMap::from_iter(EXAMPLE_FOODS.iter().map(|(k, v)| (
                BTreeSet::from_iter(k.iter().copied()),
                BTreeSet::from_iter(v.iter().copied())
            )))
        );
    }

    #[test]
    fn infer_example_mapping() {
        let foods = EXAMPLE_FOODS.iter().map(|(k, v)| {
            (
                BTreeSet::from_iter(k.iter().copied()),
                BTreeSet::from_iter(v.iter().copied()),
            )
        });
        assert_eq!(infer_mapping(foods), HashMap::from(EXAMPLE_MAPPING));
    }

    #[test]
    fn infer_custom_example_mapping() {
        let foods = CUSTOM_FOODS.iter().map(|(k, v)| {
            (
                BTreeSet::from_iter(k.iter().copied()),
                BTreeSet::from_iter(v.iter().copied()),
            )
        });
        assert_eq!(infer_mapping(foods), HashMap::from(CUSTOM_MAPPING));
    }

    #[test]
    fn list_safe_example_ingredients() {
        let foods = HashMap::from_iter(EXAMPLE_FOODS.iter().map(|(k, v)| {
            (
                BTreeSet::from_iter(k.iter().copied()),
                BTreeSet::from_iter(v.iter().copied()),
            )
        }));
        assert_eq!(
            BTreeSet::from_iter(safe_ingredients(&foods)),
            BTreeSet::from([
                Ingredient("kfcds"),
                Ingredient("nhms"),
                Ingredient("sbzzf"),
                Ingredient("sbzzf"),
                Ingredient("trh"),
            ])
        )
    }

    #[test]
    fn list_safe_custom_example_ingredients() {
        let foods = HashMap::from_iter(CUSTOM_FOODS.iter().map(|(k, v)| {
            (
                BTreeSet::from_iter(k.iter().copied()),
                BTreeSet::from_iter(v.iter().copied()),
            )
        }));
        assert_eq!(
            BTreeSet::from_iter(safe_ingredients(&foods)),
            BTreeSet::from([Ingredient("E"), Ingredient("F"), Ingredient("G"),])
        )
    }
}
