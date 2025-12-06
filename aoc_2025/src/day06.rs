use aoc_companion::prelude::*;
use itertools::Itertools as _;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Door<'input> {
    operands: ndarray::Array2<&'input str>,
    operators: Vec<Op>,
}

impl<'input> Solution<'input> for Door<'input> {
    fn parse(input: &'input str) -> Result<Self> {
        let lines: Vec<_> = input.lines().collect();
        let Some((operator_line, operand_lines)) = lines.split_last() else {
            anyhow::bail!("homework is empty");
        };
        let column_offsets: Vec<usize> = operator_line
            .as_bytes()
            .iter()
            .positions(|b| *b != b' ')
            .collect();
        let operators = operator_line
            .split_ascii_whitespace()
            .map(|op| match op {
                "+" => Ok(Op::Add),
                "*" => Ok(Op::Mul),
                _ => Err(anyhow::anyhow!("invalid operator: {op:?}")),
            })
            .try_collect()?;
        let operands = operand_lines
            .iter()
            .flat_map(|line| {
                column_offsets
                    .iter()
                    .tuple_windows()
                    .map(|(&a, &b)| &line[a..(b - 1)])
                    .chain(std::iter::once(&line[*column_offsets.last().unwrap()..]))
            })
            .collect();

        Ok(Door {
            operands: ndarray::Array2::from_shape_vec(
                (operand_lines.len(), column_offsets.len()),
                operands,
            )?,
            operators,
        })
    }

    fn part1(&self) -> Result<u64> {
        Ok(self.to_human_math()?.solutions().iter().sum())
    }

    fn part2(&self) -> Result<u64> {
        Ok(self.to_cephalopod_math()?.solutions().iter().sum())
    }
}

impl Door<'_> {
    fn to_human_math(&self) -> Result<Homework> {
        let operands = self
            .operands
            .columns()
            .into_iter()
            .map(|col| col.iter().map(|s| s.trim().parse()).try_collect())
            .try_collect()?;
        Ok(Homework {
            operands,
            operators: self.operators.clone(),
        })
    }

    fn to_cephalopod_math(&self) -> Result<Homework> {
        let operands = self
            .operands
            .columns()
            .into_iter()
            .rev()
            .map(|col| {
                let width = col.get(0).map(|s| s.len()).unwrap_or_default();
                (0..width)
                    .rev()
                    .map(|i| {
                        col.iter()
                            .map(|s| {
                                s.as_bytes()
                                    .get(i)
                                    .ok_or_else(|| anyhow::anyhow!("uneven column width"))
                            })
                            .filter_ok(|b| b.is_ascii_digit())
                            .try_fold(0, |acc, b| -> Result<u64> {
                                Ok(acc * 10 + u64::from(b? - b'0'))
                            })
                    })
                    .try_collect()
            })
            .try_collect()?;
        Ok(Homework {
            operands,
            operators: self.operators.iter().cloned().rev().collect(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Add,
    Mul,
}

impl Op {
    fn neutral_element(&self) -> u64 {
        match self {
            Op::Add => 0,
            Op::Mul => 1,
        }
    }
}

struct Homework {
    operands: Vec<Vec<u64>>,
    operators: Vec<Op>,
}

impl Homework {
    fn solutions(&self) -> Vec<u64> {
        self.operands
            .iter()
            .zip(self.operators.iter())
            .map(|(col, op)| {
                col.iter().fold(
                    op.neutral_element(),
                    match op {
                        Op::Add => std::ops::Add::add,
                        Op::Mul => std::ops::Mul::mul,
                    },
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "\
123 328  51 64 
 45 64  387 23 
  6 98  215 314
*   +   *   +  ";

    #[test]
    fn parse_example_input() {
        assert_eq!(
            Door::parse(EXAMPLE_INPUT).unwrap(),
            Door {
                operands: ndarray::array![
                    ["123", "328", " 51", "64 "],
                    [" 45", "64 ", "387", "23 "],
                    ["  6", "98 ", "215", "314"],
                ],
                operators: vec![Op::Mul, Op::Add, Op::Mul, Op::Add,]
            }
        )
    }

    #[test]
    fn example_human_solutions() {
        assert_eq!(
            Door::parse(EXAMPLE_INPUT)
                .unwrap()
                .to_human_math()
                .unwrap()
                .solutions(),
            [33210, 490, 4243455, 401]
        );
    }

    #[test]
    fn example_cephalopod_solutions() {
        assert_eq!(
            Door::parse(EXAMPLE_INPUT)
                .unwrap()
                .to_cephalopod_math()
                .unwrap()
                .solutions(),
            [1058, 3253600, 625, 8544]
        );
    }
}
