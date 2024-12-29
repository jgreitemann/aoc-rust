use std::collections::{HashMap, VecDeque};

use anyhow::{anyhow, bail};
use aoc_companion::prelude::*;
use aoc_utils::array;
use itertools::{iterate, Itertools};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Door {
    initial_state: State,
    checksum_after: usize,
    rules: HashMap<State, [Action; 2]>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        let Some((preamble, rules_str)) = input.split_once("\n\n") else {
            bail!("Missing initial blank line separating preamble from rules");
        };
        let [initial_state_str, checksum_after_str] = array::from_iter_exact(preamble.lines())
            .map_err(|lines| anyhow!("Expected two lines in preamble, got {}", lines.len()))?;
        let initial_state = State(
            initial_state_str
                .strip_prefix("Begin in state ")
                .ok_or_else(|| anyhow!("Missing initial state introducer"))?
                .strip_suffix('.')
                .ok_or_else(|| anyhow!("Missing full stop after initial state"))?
                .chars()
                .exactly_one()
                .map_err(|e| anyhow!("Expected a single character as state designator: {e}"))?,
        );
        let checksum_after = checksum_after_str
            .strip_prefix("Perform a diagnostic checksum after ")
            .ok_or_else(|| anyhow!("Missing checksum introducer"))?
            .strip_suffix(" steps.")
            .ok_or_else(|| anyhow!("Missing suffix after checksum"))?
            .parse()?;

        let rules = rules_str.split("\n\n").map(parse_rule).try_collect()?;

        Ok(Self {
            initial_state,
            checksum_after,
            rules,
        })
    }

    fn part1(&self) -> usize {
        let mut tm = TuringMachine::new();
        tm.run(self.initial_state, self.checksum_after, &self.rules);
        tm.checksum()
    }
}

fn parse_rule(input: &str) -> Result<(State, [Action; 2])> {
    let Some((state_str, actions_str)) = input.split_once("\n  If the current value is 0:\n")
    else {
        bail!("Missing rule for current value 0");
    };
    let Some((action_0_str, action_1_str)) =
        actions_str.split_once("\n  If the current value is 1:\n")
    else {
        bail!("Missing rule for current value 1");
    };
    let state = State(
        state_str
            .strip_prefix("In state ")
            .ok_or_else(|| anyhow!("Missing state introducer"))?
            .strip_suffix(':')
            .ok_or_else(|| anyhow!("Missing colon after state"))?
            .chars()
            .exactly_one()
            .map_err(|e| anyhow!("Expected a single character as state designator: {e}"))?,
    );

    Ok((state, [action_0_str.parse()?, action_1_str.parse()?]))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct State(char);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Action {
    write: bool,
    offset: isize,
    to_state: State,
}

impl std::str::FromStr for Action {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let [write_line, move_line, continue_line] = array::from_iter_exact(s.lines())
            .map_err(|lines| anyhow!("Expected three lines for action; got {}", lines.len()))?;
        let write = match write_line
            .strip_prefix("    - Write the value ")
            .ok_or_else(|| anyhow!("Missing value introducer"))?
            .strip_suffix('.')
            .ok_or_else(|| anyhow!("Missing full stop after value"))?
        {
            "0" => false,
            "1" => true,
            other => bail!("Invalid value to write: {other:?}"),
        };
        let offset = match move_line
            .strip_prefix("    - Move one slot to the ")
            .ok_or_else(|| anyhow!("Missing move introducer"))?
            .strip_suffix('.')
            .ok_or_else(|| anyhow!("Missing full stop after move direction"))?
        {
            "left" => -1,
            "right" => 1,
            other => bail!("Invalid move direction {other:?}"),
        };
        let to_state = State(
            continue_line
                .strip_prefix("    - Continue with state ")
                .ok_or_else(|| anyhow!("Missing state introducer"))?
                .strip_suffix('.')
                .ok_or_else(|| anyhow!("Missing full stop after state"))?
                .chars()
                .exactly_one()
                .map_err(|e| anyhow!("Expected a single character as state designator: {e}"))?,
        );

        Ok(Self {
            write,
            offset,
            to_state,
        })
    }
}

struct TuringMachine {
    tape: VecDeque<bool>,
    pos: usize,
}

impl TuringMachine {
    fn new() -> Self {
        Self {
            tape: VecDeque::from([false]),
            pos: 0,
        }
    }

    fn current_value(&self) -> bool {
        self.tape[self.pos]
    }

    fn write(&mut self, val: bool) {
        self.tape[self.pos] = val;
    }

    fn move_tape(&mut self, offset: isize) {
        let new_pos = self.pos as isize + offset;
        if let Ok(new_left) = (-new_pos).try_into() {
            self.tape.extend(std::iter::repeat_n(false, new_left));
            self.tape.rotate_right(new_left);
            self.pos = 0;
        } else if let Ok(new_right) = (new_pos + 1 - self.tape.len() as isize).try_into() {
            self.tape.extend(std::iter::repeat_n(false, new_right));
            self.pos = self.tape.len() - 1;
        } else {
            self.pos = new_pos as usize;
        }
    }

    fn checksum(&self) -> usize {
        self.tape.iter().filter(|&&b| b).count()
    }

    fn run(
        &mut self,
        initial_state: State,
        n: usize,
        rules: &HashMap<State, [Action; 2]>,
    ) -> State {
        iterate(initial_state, |state| {
            let Action {
                write,
                offset,
                to_state,
            } = rules[state][self.current_value() as usize];
            self.write(write);
            self.move_tape(offset);
            to_state
        })
        .nth(n - 1)
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "\
Begin in state A.
Perform a diagnostic checksum after 6 steps.

In state A:
  If the current value is 0:
    - Write the value 1.
    - Move one slot to the right.
    - Continue with state B.
  If the current value is 1:
    - Write the value 0.
    - Move one slot to the left.
    - Continue with state B.

In state B:
  If the current value is 0:
    - Write the value 1.
    - Move one slot to the left.
    - Continue with state A.
  If the current value is 1:
    - Write the value 1.
    - Move one slot to the right.
    - Continue with state A.";

    const EXAMPLE_INITIAL_STATE: State = State('A');
    const EXAMPLE_CHECKSUM_AFTER: usize = 6;
    const EXAMPLE_RULES: [(State, [Action; 2]); 2] = [
        (
            State('A'),
            [
                Action {
                    write: true,
                    offset: 1,
                    to_state: State('B'),
                },
                Action {
                    write: false,
                    offset: -1,
                    to_state: State('B'),
                },
            ],
        ),
        (
            State('B'),
            [
                Action {
                    write: true,
                    offset: -1,
                    to_state: State('A'),
                },
                Action {
                    write: true,
                    offset: 1,
                    to_state: State('A'),
                },
            ],
        ),
    ];

    #[test]
    fn parse_example_input() {
        let blueprint = Door::parse(EXAMPLE_INPUT).unwrap();
        assert_eq!(blueprint.initial_state, EXAMPLE_INITIAL_STATE);
        assert_eq!(blueprint.checksum_after, EXAMPLE_CHECKSUM_AFTER);
        assert_eq!(blueprint.rules, HashMap::from(EXAMPLE_RULES));
    }

    #[test]
    fn example_tape() {
        let mut tm = TuringMachine::new();
        tm.run(
            EXAMPLE_INITIAL_STATE,
            EXAMPLE_CHECKSUM_AFTER,
            &HashMap::from(EXAMPLE_RULES),
        );
        assert_eq!(tm.tape, [true, true, false, true]);
    }

    #[test]
    fn example_checksum() {
        let mut tm = TuringMachine::new();
        tm.run(
            EXAMPLE_INITIAL_STATE,
            EXAMPLE_CHECKSUM_AFTER,
            &HashMap::from(EXAMPLE_RULES),
        );
        assert_eq!(tm.checksum(), 3);
    }
}
