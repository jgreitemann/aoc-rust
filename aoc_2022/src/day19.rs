use aoc_companion::prelude::*;

use enum_map::{enum_map, Enum, EnumMap};
use itertools::Itertools;

use std::num::ParseIntError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Enum)]
enum Resource {
    Ore,
    Clay,
    Obsidian,
    Geode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    NoOp,
    SpendOnRobot(Resource),
}

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
        self.time_left -= 1;
        for (robot_res, count) in &self.robot_inventory.0 {
            self.resource_inventory.0[robot_res] += count;
        }
    }

    fn feasible_actions(&self, blueprint: &Blueprint) -> [Option<Action>; 5] {
        std::array::from_fn(|i| match i {
            0..=3 => {
                let robot = Resource::from_usize(i);
                self.resource_inventory
                    .can_afford_robot(robot, blueprint)
                    .then_some(Action::SpendOnRobot(robot))
            }
            4 => Some(Action::NoOp),
            _ => unreachable!(),
        })
    }

    fn spend_on_robot(&mut self, robot: Resource, blueprint: &Blueprint) {
        self.resource_inventory.spend(robot, blueprint);
        self.robot_inventory.0[robot] += 1;
    }

    fn geode_yield(&self) -> u32 {
        self.resource_inventory.0[Resource::Geode]
    }
}

fn strategy_maximizing_geode_yield(strat: Strategy, blueprint: &Blueprint) -> Strategy {
    strat
        .feasible_actions(blueprint)
        .into_iter()
        .filter_map(|action_opt| action_opt)
        .map(|action| {
            let mut new_strat = strat.clone();
            new_strat.produce();
            match action {
                Action::NoOp => {}
                Action::SpendOnRobot(robot) => new_strat.spend_on_robot(robot, blueprint),
            }

            if new_strat.time_left == 0 {
                new_strat
            } else {
                strategy_maximizing_geode_yield(new_strat, blueprint)
            }
        })
        .reduce(|lhs, rhs| std::cmp::max_by_key(lhs, rhs, Strategy::geode_yield))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_is_parsed() {
        assert_eq!(
            parse_blueprints(EXAMPLE_INPUT).unwrap(),
            example_blueprints()
        );
    }

    #[test]
    fn find_max_geode_yield() {
        let strat = strategy_maximizing_geode_yield(Strategy::new(), &example_blueprints()[0]);
        assert_eq!(strat.geode_yield(), 9);
    }

    const EXAMPLE_INPUT: &str = "\
        Blueprint 1: \
          Each ore robot costs 4 ore. \
          Each clay robot costs 2 ore. \
          Each obsidian robot costs 3 ore and 14 clay. \
          Each geode robot costs 2 ore and 7 obsidian.\n\
        Blueprint 2: \
          Each ore robot costs 2 ore. \
          Each clay robot costs 3 ore. \
          Each obsidian robot costs 3 ore and 8 clay. \
          Each geode robot costs 3 ore and 12 obsidian.";

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
