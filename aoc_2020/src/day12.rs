use aoc_companion::prelude::*;
use aoc_utils::linalg::Vector;
use itertools::Itertools;

pub(crate) struct Door {
    instructions: Vec<Instruction>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseError> {
        let instructions = input.lines().map(|line| line.parse()).try_collect()?;
        Ok(Door { instructions })
    }

    fn part1(&self) -> i32 {
        final_state::<ShipState>(&self.instructions)
            .position
            .norm_l1()
    }

    fn part2(&self) -> i32 {
        final_state::<WaypointState>(&self.instructions)
            .position
            .norm_l1()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Instruction {
    MoveBy { delta: Vector<i32, 2> },
    Turn { quadrants: i32 },
    MoveForward { amount: i32 },
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ParseError {
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("first instruction byte is multibyte")]
    MultibyteInstruction,
    #[error("invalid instruction type {0:?}")]
    InvalidInstructionType(char),
    #[error("angle of {0} degrees is not axis-aligned")]
    AngleNotAxisAligned(i32),
}

fn parse_quadrants(s: &str) -> Result<i32, ParseError> {
    let angle = s.parse()?;
    if angle % 90 == 0 {
        Ok(angle / 90)
    } else {
        Err(ParseError::AngleNotAxisAligned(angle))
    }
}

impl std::str::FromStr for Instruction {
    type Err = ParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let (c, rest) = s
            .split_at_checked(1)
            .ok_or(ParseError::MultibyteInstruction)?;
        Ok(match c {
            "E" => Instruction::MoveBy {
                delta: Vector([rest.parse()?, 0]),
            },
            "N" => Instruction::MoveBy {
                delta: Vector([0, rest.parse()?]),
            },
            "W" => Instruction::MoveBy {
                delta: Vector([-rest.parse()?, 0]),
            },
            "S" => Instruction::MoveBy {
                delta: Vector([0, -rest.parse()?]),
            },
            "L" => Instruction::Turn {
                quadrants: parse_quadrants(rest)?,
            },
            "R" => Instruction::Turn {
                quadrants: -parse_quadrants(rest)?,
            },
            "F" => Instruction::MoveForward {
                amount: rest.parse()?,
            },
            _ => Err(ParseError::InvalidInstructionType(
                c.chars().next().unwrap(),
            ))?,
        })
    }
}

trait State {
    fn turn(&mut self, quadrants: i32);
    fn move_by(&mut self, delta: Vector<i32, 2>);
    fn move_forward(&mut self, amount: i32);

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::MoveBy { delta } => self.move_by(delta),
            Instruction::Turn { quadrants } => self.turn(quadrants),
            Instruction::MoveForward { amount } => self.move_forward(amount),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShipState {
    facing: Vector<i32, 2>,
    position: Vector<i32, 2>,
}

impl Default for ShipState {
    fn default() -> Self {
        Self {
            facing: Vector([1, 0]),
            position: Vector::default(),
        }
    }
}

impl State for ShipState {
    fn turn(&mut self, quadrants: i32) {
        self.facing = rotated(self.facing, quadrants);
    }

    fn move_by(&mut self, delta: Vector<i32, 2>) {
        self.position += delta;
    }

    fn move_forward(&mut self, amount: i32) {
        self.move_by(self.facing * amount);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WaypointState {
    waypoint: Vector<i32, 2>,
    position: Vector<i32, 2>,
}

impl Default for WaypointState {
    fn default() -> Self {
        Self {
            waypoint: Vector([10, 1]),
            position: Vector::default(),
        }
    }
}

impl State for WaypointState {
    fn turn(&mut self, quadrants: i32) {
        self.waypoint = rotated(self.waypoint, quadrants);
    }

    fn move_by(&mut self, delta: Vector<i32, 2>) {
        self.waypoint += delta;
    }

    fn move_forward(&mut self, amount: i32) {
        self.position += self.waypoint * amount;
    }
}

fn rotated(mut v: Vector<i32, 2>, quadrants: i32) -> Vector<i32, 2> {
    if (quadrants + 4) % 2 != 0 {
        v = Vector([-v[1], v[0]]);
    }
    if (quadrants + 4) / 2 % 2 != 0 {
        v *= -1;
    }
    v
}

fn final_state<S: State + Default>(instructions: &[Instruction]) -> S {
    instructions
        .iter()
        .fold(S::default(), |mut state, &instruction| {
            state.execute(instruction);
            state
        })
}

#[cfg(test)]
mod tests {
    use itertools::assert_equal;

    use super::*;

    const EXAMPLE_INPUT: &str = "\
F10
N3
F7
R90
F11";

    const EXAMPLE_INSTRUCTIONS: &[Instruction] = &[
        Instruction::MoveForward { amount: 10 },
        Instruction::MoveBy {
            delta: Vector([0, 3]),
        },
        Instruction::MoveForward { amount: 7 },
        Instruction::Turn { quadrants: -1 },
        Instruction::MoveForward { amount: 11 },
    ];

    #[test]
    fn parse_example_input() {
        assert_eq!(
            EXAMPLE_INPUT
                .lines()
                .map(|line| line.parse().unwrap())
                .collect::<Vec<Instruction>>(),
            EXAMPLE_INSTRUCTIONS
        );
    }

    #[test]
    fn intermediate_ship_states_in_example() {
        assert_equal(
            EXAMPLE_INSTRUCTIONS
                .iter()
                .scan(ShipState::default(), |state, &instruction| {
                    state.execute(instruction);
                    Some(state.clone())
                }),
            [
                ShipState {
                    facing: Vector([1, 0]),
                    position: Vector([10, 0]),
                },
                ShipState {
                    facing: Vector([1, 0]),
                    position: Vector([10, 3]),
                },
                ShipState {
                    facing: Vector([1, 0]),
                    position: Vector([17, 3]),
                },
                ShipState {
                    facing: Vector([0, -1]),
                    position: Vector([17, 3]),
                },
                ShipState {
                    facing: Vector([0, -1]),
                    position: Vector([17, -8]),
                },
            ],
        );
    }

    #[test]
    fn final_ship_state_after_example_instructions() {
        assert_eq!(
            final_state::<ShipState>(EXAMPLE_INSTRUCTIONS),
            ShipState {
                facing: Vector([0, -1]),
                position: Vector([17, -8])
            }
        )
    }

    #[test]
    fn intermediate_waypoint_states_in_example() {
        assert_equal(
            EXAMPLE_INSTRUCTIONS
                .iter()
                .scan(WaypointState::default(), |state, &instruction| {
                    state.execute(instruction);
                    Some(state.clone())
                }),
            [
                WaypointState {
                    waypoint: Vector([10, 1]),
                    position: Vector([100, 10]),
                },
                WaypointState {
                    waypoint: Vector([10, 4]),
                    position: Vector([100, 10]),
                },
                WaypointState {
                    waypoint: Vector([10, 4]),
                    position: Vector([170, 38]),
                },
                WaypointState {
                    waypoint: Vector([4, -10]),
                    position: Vector([170, 38]),
                },
                WaypointState {
                    waypoint: Vector([4, -10]),
                    position: Vector([214, -72]),
                },
            ],
        );
    }

    #[test]
    fn final_waypoint_state_after_example_instructions() {
        assert_eq!(
            final_state::<WaypointState>(EXAMPLE_INSTRUCTIONS),
            WaypointState {
                waypoint: Vector([4, -10]),
                position: Vector([214, -72]),
            }
        )
    }
}
