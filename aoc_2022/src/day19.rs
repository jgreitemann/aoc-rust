use aoc_companion::prelude::*;

use enum_map::{enum_map, Enum, EnumMap};
use genawaiter::sync::GenBoxed;
use itertools::Itertools;

use std::num::ParseIntError;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Enum)]
enum Resource {
    Ore,
    Clay,
    Obsidian,
    Geode,
}

const RESOURCES: &[Resource] = &[
    Resource::Ore,
    Resource::Clay,
    Resource::Obsidian,
    Resource::Geode,
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct Costs(EnumMap<Resource, u32>);

#[derive(Debug, Clone, PartialEq, Eq)]
struct Blueprint {
    robot_costs: EnumMap<Resource, Costs>,
}

fn parse_blueprints(input: &str) -> Result<Vec<Blueprint>, ParseIntError> {
    use Resource::*;
    let re = regex::Regex::new(r"Blueprint \d+: Each ore robot costs (?P<ore_ore>\d+) ore. Each clay robot costs (?P<clay_ore>\d+) ore. Each obsidian robot costs (?P<obs_ore>\d+) ore and (?P<obs_clay>\d+) clay. Each geode robot costs (?P<geode_ore>\d+) ore and (?P<geode_obs>\d+) obsidian.").unwrap();
    re.captures_iter(input)
        .map(|capt| {
            Ok(Blueprint {
                robot_costs: enum_map! {
                       Ore => Costs(enum_map!{
                           Ore => capt["ore_ore"].parse()?,
                           _ => 0
                       }),
                       Clay => Costs(enum_map!{
                           Ore => capt["clay_ore"].parse()?,
                           _ => 0
                       }),
                       Obsidian => Costs(enum_map!{
                           Ore => capt["obs_ore"].parse()?,
                           Clay => capt["obs_clay"].parse()?,
                           _ => 0
                       }),
                       Geode => Costs(enum_map!{
                           Ore => capt["geode_ore"].parse()?,
                           Obsidian => capt["geode_obs"].parse()?,
                           _ => 0
                       }),
                },
            })
        })
        .try_collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Inventory(EnumMap<Resource, u32>);

impl Inventory {
    fn can_afford_robot(&self, robot: Resource, blueprint: &Blueprint) -> bool {
        let costs = &blueprint.robot_costs[robot];
        costs.0.iter().all(|(res, cost)| *cost <= self.0[res])
    }

    fn spend(&mut self, robot: Resource, blueprint: &Blueprint) {
        let costs = &blueprint.robot_costs[robot];
        for (res, cost) in &costs.0 {
            self.0[res] -= cost;
        }
    }
}

#[derive(Debug, Clone)]
struct Strategy {
    resource_inventory: Inventory,
    robot_inventory: Inventory,
    time_left: u32,
}

impl Strategy {
    fn new() -> Self {
        Strategy {
            resource_inventory: Inventory(EnumMap::default()),
            robot_inventory: Inventory(enum_map! {
                Resource::Ore => 1,
                _ => 0,
            }),
            time_left: 20,
        }
    }

    fn produce(&mut self) {
        for (robot_res, count) in &self.robot_inventory.0 {
            self.resource_inventory.0[robot_res] += count;
        }
    }

    fn robot_options(&self, blueprint: Arc<Blueprint>) -> impl Iterator<Item = Resource> + '_ {
        RESOURCES
            .iter()
            .copied()
            .filter(move |robot| self.resource_inventory.can_afford_robot(*robot, &blueprint))
    }

    fn spend_on_robot(&mut self, robot: Resource, blueprint: &Blueprint) {
        self.resource_inventory.spend(robot, blueprint);
        self.robot_inventory.0[robot] += 1;
    }

    fn geode_yield(&self) -> u32 {
        self.resource_inventory.0[Resource::Geode]
    }
}

fn strategy_dfs(
    mut strat: Strategy,
    blueprint: Arc<Blueprint>,
) -> genawaiter::sync::GenBoxed<Strategy> {
    strat.time_left -= 1;

    GenBoxed::new_boxed(|co| async move {
        for robot in strat.robot_options(blueprint.clone()) {
            let mut new_strat = strat.clone();
            new_strat.produce();
            new_strat.spend_on_robot(robot, &blueprint);

            if new_strat.time_left == 0 {
                co.yield_(new_strat).await;
            } else {
                for s in strategy_dfs(new_strat, blueprint.clone()) {
                    co.yield_(s).await;
                }
            }
        }

        // do nothing
        strat.produce();
        if strat.time_left == 0 {
            co.yield_(strat).await;
        } else {
            for s in strategy_dfs(strat, blueprint.clone()) {
                co.yield_(s).await;
            }
        }
    })
}

fn strategy_maximizing_geode_yield(blueprint: &Blueprint) -> Strategy {
    strategy_dfs(Strategy::new(), Arc::new(blueprint.clone()))
        .into_iter()
        .max_by_key(|strat| strat.geode_yield())
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_max_geode_yield() {
        let strat = strategy_maximizing_geode_yield(&example_blueprints()[0]);
        assert_eq!(strat.geode_yield(), 9);
    }

    fn example_blueprints() -> Vec<Blueprint> {
        use Resource::*;
        vec![
            Blueprint {
                robot_costs: enum_map! {
                    Ore => Costs(enum_map!{
                        Ore => 4,
                        _ => 0
                    }),
                   Clay => Costs(enum_map!{
                       Ore => 2,
                       _ => 0
                   }),
                   Obsidian => Costs(enum_map!{
                       Ore => 3,
                       Clay => 14,
                       _ => 0
                   }),
                   Geode => Costs(enum_map!{
                       Ore => 2,
                       Obsidian => 7,
                       _ => 0
                   }),
                },
            },
            Blueprint {
                robot_costs: enum_map! {
                   Ore => Costs(enum_map!{
                        Ore => 2,
                        _ => 0
                   }),
                   Clay => Costs(enum_map!{
                       Ore => 3,
                       _ => 0
                   }),
                   Obsidian => Costs(enum_map!{
                       Ore => 3,
                       Clay => 8,
                       _ => 0
                   }),
                   Geode => Costs(enum_map!{
                       Ore => 3,
                       Obsidian => 12,
                       _ => 0
                   }),
                },
            },
        ]
    }
}
