use aoc_companion::prelude::*;

use genawaiter::sync::GenBoxed;
use itertools::Itertools;
use thiserror::Error;

use std::collections::HashMap;
use std::num::ParseIntError;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
struct Costs(HashMap<Resource, u32>);

#[derive(Debug, Clone, PartialEq, Eq)]
struct Blueprint {
    robot_costs: HashMap<Resource, Costs>,
}

fn parse_blueprints(input: &str) -> Result<Vec<Blueprint>, ParseIntError> {
    use Resource::*;
    let re = regex::Regex::new(r"Blueprint \d+: Each ore robot costs (?P<ore_ore>\d+) ore. Each clay robot costs (?P<clay_ore>\d+) ore. Each obsidian robot costs (?P<obs_ore>\d+) ore and (?P<obs_clay>\d+) clay. Each geode robot costs (?P<geode_ore>\d+) ore and (?P<geode_obs>\d+) obsidian.").unwrap();
    re.captures_iter(input)
        .map(|capt| {
            Ok(Blueprint {
                robot_costs: HashMap::from([
                    (Ore, Costs(HashMap::from([(Ore, capt["ore_ore"].parse()?)]))),
                    (
                        Clay,
                        Costs(HashMap::from([(Ore, capt["clay_ore"].parse()?)])),
                    ),
                    (
                        Obsidian,
                        Costs(HashMap::from([
                            (Ore, capt["obs_ore"].parse()?),
                            (Clay, capt["obs_clay"].parse()?),
                        ])),
                    ),
                    (
                        Geode,
                        Costs(HashMap::from([
                            (Ore, capt["geode_ore"].parse()?),
                            (Obsidian, capt["geode_obs"].parse()?),
                        ])),
                    ),
                ]),
            })
        })
        .try_collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Inventory(HashMap<Resource, u32>);

impl Inventory {
    fn can_afford_robot(&self, robot: Resource, blueprint: &Blueprint) -> bool {
        let costs = &blueprint.robot_costs[&robot];
        costs
            .0
            .iter()
            .all(|(res, cost)| cost <= self.0.get(res).unwrap_or(&0))
    }

    fn spend(&mut self, robot: Resource, blueprint: &Blueprint) {
        let costs = &blueprint.robot_costs[&robot];
        for (res, cost) in &costs.0 {
            *self.0.get_mut(res).unwrap() -= cost;
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
            resource_inventory: Inventory(HashMap::new()),
            robot_inventory: Inventory(HashMap::from([(Resource::Ore, 1)])),
            time_left: 19,
        }
    }

    fn produce(&mut self) {
        for (robot_res, count) in &self.robot_inventory.0 {
            let existing = self.resource_inventory.0.get(robot_res).unwrap_or(&0);
            self.resource_inventory
                .0
                .insert(*robot_res, existing + count);
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
        let existing_robots = self.robot_inventory.0.get(&robot).copied().unwrap_or(0);
        self.robot_inventory.0.insert(robot, existing_robots + 1);
    }

    fn geode_yield(&self) -> u32 {
        self.resource_inventory
            .0
            .get(&Resource::Geode)
            .copied()
            .unwrap_or(0)
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
        // Blueprint 1:
        //  Each ore robot costs 4 ore.
        //  Each clay robot costs 2 ore.
        //  Each obsidian robot costs 3 ore and 14 clay.
        //  Each geode robot costs 2 ore and 7 obsidian.
        //
        // Blueprint 2:
        //  Each ore robot costs 2 ore.
        //  Each clay robot costs 3 ore.
        //  Each obsidian robot costs 3 ore and 8 clay.
        //  Each geode robot costs 3 ore and 12 obsidian.
        use Resource::*;
        vec![
            Blueprint {
                robot_costs: HashMap::from([
                    (Ore, Costs(HashMap::from([(Ore, 4)]))),
                    (Clay, Costs(HashMap::from([(Ore, 2)]))),
                    (Obsidian, Costs(HashMap::from([(Ore, 3), (Clay, 14)]))),
                    (Geode, Costs(HashMap::from([(Ore, 2), (Obsidian, 7)]))),
                ]),
            },
            Blueprint {
                robot_costs: HashMap::from([
                    (Ore, Costs(HashMap::from([(Ore, 2)]))),
                    (Clay, Costs(HashMap::from([(Ore, 3)]))),
                    (Obsidian, Costs(HashMap::from([(Ore, 3), (Clay, 8)]))),
                    (Geode, Costs(HashMap::from([(Ore, 3), (Obsidian, 12)]))),
                ]),
            },
        ]
    }
}
