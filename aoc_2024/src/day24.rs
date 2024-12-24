use std::collections::HashMap;

use anyhow::bail;
use aoc_companion::prelude::*;
use itertools::Itertools;

pub(crate) struct Door {
    initial_wires: HashMap<String, bool>,
    gates: GatesMap,
}

type GatesMap = HashMap<String, Vec<(String, Gate, String)>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Gate {
    And,
    Or,
    Xor,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        let Some((init_str, gates_str)) = input.split_once("\n\n") else {
            bail!("Missing empty line separating sections");
        };
        let initial_wires = init_str
            .lines()
            .map(|line| {
                let Some((wire_name, wire_val_str)) = line.split_once(": ") else {
                    panic!("Missing colon between wire name and initial value");
                };
                (
                    wire_name.to_string(),
                    match wire_val_str {
                        "0" => false,
                        "1" => true,
                        _ => unreachable!(),
                    },
                )
            })
            .collect();

        let gates: Vec<(String, (String, Gate, String))> = gates_str
            .lines()
            .map(|line| {
                let Some((op_str, res_str)) = line.split_once(" -> ") else {
                    panic!("Missing arrow in gate line");
                };
                if let Some((lhs, rhs)) = op_str.split_once(" AND ") {
                    (lhs, Gate::And, rhs, res_str)
                } else if let Some((lhs, rhs)) = op_str.split_once(" OR ") {
                    (lhs, Gate::Or, rhs, res_str)
                } else if let Some((lhs, rhs)) = op_str.split_once(" XOR ") {
                    (lhs, Gate::Xor, rhs, res_str)
                } else {
                    unreachable!()
                }
            })
            .flat_map(|(lhs, gate, rhs, res)| {
                [
                    (lhs.to_string(), (rhs.to_string(), gate, res.to_string())),
                    (rhs.to_string(), (lhs.to_string(), gate, res.to_string())),
                ]
            })
            .collect();

        Ok(Self {
            initial_wires,
            gates: gates.into_iter().into_group_map(),
        })
    }

    fn part1(&self) -> u64 {
        let outputs = self.produce_outputs();
        read_numeric_output(&outputs)
    }

    fn part2(&self) -> String {
        let mut crossed_wires = ripple_carry_defects(&self.gates).into_keys().collect_vec();
        crossed_wires.sort();
        crossed_wires.join(",")
    }
}

impl Gate {
    fn exec(&self, lhs: bool, rhs: bool) -> bool {
        match self {
            Gate::And => lhs & rhs,
            Gate::Or => lhs | rhs,
            Gate::Xor => lhs ^ rhs,
        }
    }
}

impl Door {
    fn produce_outputs(&self) -> HashMap<String, bool> {
        let mut outputs = self.initial_wires.clone();
        let mut todo = self.initial_wires.keys().cloned().collect_vec();
        while let Some(wire) = todo.pop() {
            for (other, gate, out) in self.gates.get(&wire).into_iter().flatten() {
                if let Some(&other_val) = outputs.get(other) {
                    let this_val = *outputs.get(&wire).unwrap();
                    let res = gate.exec(this_val, other_val);
                    outputs.insert(out.clone(), res);
                    todo.push(out.clone());
                }
            }
        }

        outputs
    }
}

fn read_numeric_output(outputs: &HashMap<String, bool>) -> u64 {
    (0..64)
        .filter_map(|i| outputs.get(&format!("z{i:02}")).map(|&out| (i, out)))
        .fold(0, |num, (i, bit)| num | ((bit as u64) << i))
}

type DefectsMap = HashMap<String, String>;

fn ripple_carry_defects(gates: &GatesMap) -> DefectsMap {
    let (final_carry, defects) = (1..=44)
        .fold(half_adder_defects(gates), |(carry_in, defects), i| {
            full_adder_defects(i, &carry_in, gates, defects)
        });
    assert_eq!(final_carry, "z45");
    defects
}

fn half_adder_defects(gates: &GatesMap) -> (String, DefectsMap) {
    let mut defects = DefectsMap::new();
    let expected_z0 = gate_out("x00", "y00", Gate::Xor, gates);
    expect_eq(&expected_z0, "z00", &mut defects);

    (gate_out("x00", "y00", Gate::And, gates), defects)
}

fn full_adder_defects(
    i: usize,
    carry_in: &str,
    gates: &GatesMap,
    mut defects: DefectsMap,
) -> (String, DefectsMap) {
    let x = format!("x{i:02}");
    let y = format!("y{i:02}");
    let z = format!("z{i:02}");

    let a = gate_out(&x, &y, Gate::Xor, gates);
    let d = gate_out(&x, &y, Gate::And, gates);

    match checked_gate_out(&a, carry_in, Gate::Xor, gates) {
        Some(expected_z) => {
            expect_eq(&expected_z, &z, &mut defects);
        }
        None => {
            if let Some(expected_z) = checked_gate_out(&d, carry_in, Gate::Xor, gates) {
                if expected_z == z {
                    defects.insert(a.clone(), d.clone());
                    defects.insert(d.clone(), a.clone());
                } else {
                    panic!("swapping a={a:?} and d={d:?} didn't help: gate exists, but still doesn't produce {z:?}; instead produced {expected_z:?}");
                }
            } else {
                panic!("swapping a={a:?} and d={d:?} didn't help: no XOR gate between d={d:?} and c={carry_in:?} either");
            }
        }
    }

    let b = fixed(
        gate_out(&fixed(a, &defects), carry_in, Gate::And, gates),
        &defects,
    );
    let carry_out = fixed(gate_out(&b, &fixed(d, &defects), Gate::Or, gates), &defects);

    (carry_out, defects)
}

fn expect_eq(lhs: &str, rhs: &str, defects: &mut DefectsMap) {
    if lhs != rhs {
        defects.insert(lhs.to_string(), rhs.to_string());
        defects.insert(rhs.to_string(), lhs.to_string());
    }
}

fn fixed(x: String, defects: &DefectsMap) -> String {
    defects.get(&x).map(|s| s.to_string()).unwrap_or(x)
}

fn checked_gate_out(a: &str, b: &str, gate: Gate, gates: &GatesMap) -> Option<String> {
    gates
        .get(a)?
        .iter()
        .find_map(|(other, g, out)| (other == b && *g == gate).then_some(out))
        .cloned()
}

fn gate_out(a: &str, b: &str, gate: Gate, gates: &GatesMap) -> String {
    checked_gate_out(a, b, gate, gates)
        .unwrap_or_else(|| panic!("expected {a:?} and {b:?} to be input for {gate:?} gate"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "\
x00: 1
x01: 0
x02: 1
x03: 1
x04: 0
y00: 1
y01: 1
y02: 1
y03: 1
y04: 1

ntg XOR fgs -> mjb
y02 OR x01 -> tnw
kwq OR kpj -> z05
x00 OR x03 -> fst
tgd XOR rvg -> z01
vdt OR tnw -> bfw
bfw AND frj -> z10
ffh OR nrd -> bqk
y00 AND y03 -> djm
y03 OR y00 -> psh
bqk OR frj -> z08
tnw OR fst -> frj
gnj AND tgd -> z11
bfw XOR mjb -> z00
x03 OR x00 -> vdt
gnj AND wpb -> z02
x04 AND y00 -> kjc
djm OR pbm -> qhw
nrd AND vdt -> hwm
kjc AND fst -> rvg
y04 OR y02 -> fgs
y01 AND x02 -> pbm
ntg OR kjc -> kwq
psh XOR fgs -> tgd
qhw XOR tgd -> z09
pbm OR djm -> kpj
x03 XOR y03 -> ffh
x00 XOR y04 -> ntg
bfw OR bqk -> z06
nrd XOR fgs -> wpb
frj XOR qhw -> z04
bqk OR frj -> z07
y03 OR x01 -> nrd
hwm AND bqk -> z03
tgd XOR rvg -> z12
tnw OR pbm -> gnj";

    #[test]
    fn example_numeric_output() {
        let door = Door::parse(EXAMPLE_INPUT).unwrap();
        let outputs = door.produce_outputs();
        assert_eq!(read_numeric_output(&outputs), 2024);
    }
}
