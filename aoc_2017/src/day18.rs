use std::{
    cell::Cell,
    collections::{hash_map, HashMap},
    num::ParseIntError,
    str::FromStr,
    sync::{atomic::AtomicI64, Arc},
};
use tracing::{info, info_span, instrument};

use aoc_companion::prelude::*;

pub(crate) struct Door {
    asm: Vec<Instruction>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ParseError {
    #[error("expected a space delimiting tokens in instruction")]
    MissingToken,
    #[error("invalid instruction {0:?}")]
    InvalidInstruction(String),
    #[error("invalid immediate value")]
    InvalidImmediate(#[from] ParseIntError),
    #[error("invalid register {0:?}")]
    InvalidRegister(String),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum RuntimeError {
    #[error("executing the rcv instruction before any snd instruction has been executed")]
    NoSnd,
    #[error("program terminated due to jump to PC {0}")]
    InvalidJump(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Instruction {
    Snd(Operand),
    Set(Register, Operand),
    Add(Register, Operand),
    Mul(Register, Operand),
    Mod(Register, Operand),
    Rcv(Register),
    Jgz(Operand, Operand),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Operand {
    Immediate(i64),
    Register(Register),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Register(pub u8);

impl std::fmt::Debug for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", std::str::from_utf8(&[self.0]).unwrap_or("�"))
    }
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self, ParseError> {
        parse_assembly(input).map(|asm| Door { asm })
    }

    fn part1(&self) -> Result<i64, RuntimeError> {
        let last_value = LastValue::default();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        runtime.block_on(run(HashMap::new(), &self.asm, &last_value, &last_value))
    }

    fn part2(&self) -> Result<i64, RuntimeError> {
        let pending_recv_count = Arc::new(AtomicI64::default());
        let (send_0, recv_0) = channel(pending_recv_count.clone());
        let (send_1, recv_1) = channel(pending_recv_count);
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let (res_0, _) = runtime.block_on(async {
            tokio::try_join!(
                info_span!("program 0").in_scope(|| {
                    run(
                        HashMap::from([(Register(b'p'), 0)]),
                        &self.asm,
                        send_0,
                        recv_1,
                    )
                }),
                info_span!("program 1").in_scope(|| {
                    run(
                        HashMap::from([(Register(b'p'), 1)]),
                        &self.asm,
                        send_1,
                        recv_0,
                    )
                })
            )
        })?;
        Ok(res_0)
    }
}

fn parse_assembly(input: &str) -> Result<Vec<Instruction>, ParseError> {
    input.lines().map(str::parse).collect()
}

impl FromStr for Instruction {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, args) = s.split_once(' ').ok_or(ParseError::MissingToken)?;

        Ok(match name {
            "snd" | "rcv" => match name {
                "snd" => Instruction::Snd(args.parse()?),
                "rcv" => Instruction::Rcv(args.parse()?),
                _ => unreachable!(),
            },
            "set" | "add" | "mul" | "mod" => {
                let (reg, op) = args.split_once(' ').ok_or(ParseError::MissingToken)?;
                let reg = reg.parse()?;
                let op = op.parse()?;
                match name {
                    "set" => Instruction::Set(reg, op),
                    "add" => Instruction::Add(reg, op),
                    "mul" => Instruction::Mul(reg, op),
                    "mod" => Instruction::Mod(reg, op),
                    _ => unreachable!(),
                }
            }
            "jgz" => {
                let (cond, op) = args.split_once(' ').ok_or(ParseError::MissingToken)?;
                let cond = cond.parse()?;
                let op = op.parse()?;
                Instruction::Jgz(cond, op)
            }
            _ => return Err(ParseError::InvalidInstruction(name.to_string())),
        })
    }
}

impl FromStr for Operand {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.as_bytes() {
            [byte @ b'a'..=b'z'] => Operand::Register(Register(*byte)),
            _ => Operand::Immediate(s.parse()?),
        })
    }
}

impl FromStr for Register {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let [byte @ b'a'..=b'z'] = s.as_bytes() {
            Ok(Register(*byte))
        } else {
            Err(ParseError::InvalidRegister(s.to_string()))
        }
    }
}

impl Register {
    pub(crate) fn access<'a>(
        &self,
        registers: &'a mut HashMap<Register, i64>,
    ) -> hash_map::Entry<'a, Register, i64> {
        registers.entry(*self)
    }
}

impl Operand {
    pub(crate) fn fetch(&self, registers: &mut HashMap<Register, i64>) -> i64 {
        match self {
            Operand::Immediate(val) => *val,
            Operand::Register(reg) => *reg.access(registers).or_default(),
        }
    }
}

type RegisterAccess<'a> = hash_map::Entry<'a, Register, i64>;

enum ReceiveResult {
    Ok,
    Interrupt(i64),
    Err(RuntimeError),
}

trait Sender {
    async fn send(&self, val: i64) -> Result<(), RuntimeError>;
}

trait Receiver {
    async fn recv(&mut self, reg: RegisterAccess<'_>) -> ReceiveResult;
}

#[derive(Debug, Default)]
struct LastValue(Cell<Option<i64>>);

impl Sender for &LastValue {
    async fn send(&self, val: i64) -> Result<(), RuntimeError> {
        self.0.set(Some(val));
        Ok(())
    }
}

impl Receiver for &LastValue {
    async fn recv(&mut self, reg: RegisterAccess<'_>) -> ReceiveResult {
        if *reg.or_default() != 0 {
            self.0.take().map_or(
                ReceiveResult::Err(RuntimeError::NoSnd),
                ReceiveResult::Interrupt,
            )
        } else {
            ReceiveResult::Ok
        }
    }
}

#[derive(Debug)]
struct ChannelSender {
    tx: tokio::sync::mpsc::Sender<i64>,
    send_count: Arc<AtomicI64>,
    pending_recv_count: Arc<AtomicI64>,
}

impl Sender for ChannelSender {
    #[instrument]
    async fn send(&self, val: i64) -> Result<(), RuntimeError> {
        info!(val, "send");
        let _ = self.tx.send(val).await;
        self.send_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.pending_recv_count
            .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
}

#[derive(Debug)]
struct ChannelReceiver {
    rx: tokio::sync::mpsc::Receiver<i64>,
    send_count: Arc<AtomicI64>,
    pending_recv_count: Arc<AtomicI64>,
}

impl Receiver for ChannelReceiver {
    #[instrument]
    async fn recv(&mut self, reg: RegisterAccess<'_>) -> ReceiveResult {
        let concurrent_recv = self
            .pending_recv_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if concurrent_recv > 0 {
            return ReceiveResult::Interrupt(
                self.send_count.load(std::sync::atomic::Ordering::SeqCst),
            );
        }
        info!(%concurrent_recv, "recv");
        let Some(val) = self.rx.recv().await else {
            return ReceiveResult::Interrupt(
                self.send_count.load(std::sync::atomic::Ordering::SeqCst),
            );
        };
        info!(val, "received");
        *reg.or_default() = val;
        ReceiveResult::Ok
    }
}

fn channel(pending_recv_count: Arc<AtomicI64>) -> (ChannelSender, ChannelReceiver) {
    let send_count = Arc::new(AtomicI64::default());
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    (
        ChannelSender {
            tx,
            send_count: send_count.clone(),
            pending_recv_count: pending_recv_count.clone(),
        },
        ChannelReceiver {
            rx,
            send_count,
            pending_recv_count,
        },
    )
}

async fn run(
    mut registers: HashMap<Register, i64>,
    asm: &[Instruction],
    sender: impl Sender,
    mut receiver: impl Receiver,
) -> Result<i64, RuntimeError> {
    let mut pc = 0;
    loop {
        let mut jump = 1;
        match asm[pc] {
            Instruction::Snd(op) => sender.send(op.fetch(&mut registers)).await?,
            Instruction::Set(reg, op) => {
                let val = op.fetch(&mut registers);
                registers.insert(reg, val);
            }
            Instruction::Add(reg, op) => {
                let op_val = op.fetch(&mut registers);
                *reg.access(&mut registers).or_default() += op_val;
            }
            Instruction::Mul(reg, op) => {
                let op_val = op.fetch(&mut registers);
                *reg.access(&mut registers).or_default() *= op_val;
            }
            Instruction::Mod(reg, op) => {
                let op_val = op.fetch(&mut registers);
                *reg.access(&mut registers).or_default() %= op_val;
            }
            Instruction::Rcv(reg) => match receiver.recv(reg.access(&mut registers)).await {
                ReceiveResult::Ok => {}
                ReceiveResult::Interrupt(return_val) => return Ok(return_val),
                ReceiveResult::Err(e) => return Err(e),
            },
            Instruction::Jgz(cond, op) => {
                if cond.fetch(&mut registers) > 0 {
                    jump = op.fetch(&mut registers);
                }
            }
        }

        let new_pc = pc as i64 + jump;
        if (0..asm.len() as i64).contains(&new_pc) {
            pc = new_pc as usize;
        } else {
            return Err(RuntimeError::InvalidJump(new_pc));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use tracing_test::traced_test;

    const EXAMPLE_INPUT: &str = "set a 1
add a 2
mul a a
mod a 5
snd a
set a 0
rcv a
jgz a -1
set a 1
jgz a -2";

    const THIRD_PARTY_INPUT: &str = "set i 31
set a 1
mul p 17
jgz p p
mul a 2
add i -1
jgz i -2
add a -1
set i 127
set p 622
mul p 8505
mod p a
mul p 129749
add p 12345
mod p a
set b p
mod b 10000
snd b
add i -1
jgz i -9
jgz a 3
rcv b
jgz b -1
set f 0
set i 126
rcv a
rcv b
set p a
mul p -1
add p b
jgz p 4
snd a
set a b
jgz 1 3
snd b
set f 1
add i -1
jgz i -11
snd a
jgz f -16
jgz a -19";

    const EXAMPLE_ASM: [Instruction; 10] = [
        Instruction::Set(Register(b'a'), Operand::Immediate(1)),
        Instruction::Add(Register(b'a'), Operand::Immediate(2)),
        Instruction::Mul(Register(b'a'), Operand::Register(Register(b'a'))),
        Instruction::Mod(Register(b'a'), Operand::Immediate(5)),
        Instruction::Snd(Operand::Register(Register(b'a'))),
        Instruction::Set(Register(b'a'), Operand::Immediate(0)),
        Instruction::Rcv(Register(b'a')),
        Instruction::Jgz(Operand::Register(Register(b'a')), Operand::Immediate(-1)),
        Instruction::Set(Register(b'a'), Operand::Immediate(1)),
        Instruction::Jgz(Operand::Register(Register(b'a')), Operand::Immediate(-2)),
    ];

    const DEADLOCK_EXAMPLE_ASM: [Instruction; 7] = [
        Instruction::Snd(Operand::Immediate(1)),
        Instruction::Snd(Operand::Immediate(2)),
        Instruction::Snd(Operand::Register(Register(b'p'))),
        Instruction::Rcv(Register(b'a')),
        Instruction::Rcv(Register(b'b')),
        Instruction::Rcv(Register(b'c')),
        Instruction::Rcv(Register(b'd')),
    ];

    #[test]
    fn example_input_is_parsed_correctly() {
        assert_eq!(parse_assembly(EXAMPLE_INPUT).unwrap(), EXAMPLE_ASM);
    }

    proptest! {
        #[test]
        fn parsing_assembly_does_not_panic(input in r"\PC*") {
            let _ = parse_assembly(&input);
        }
    }

    #[test]
    fn example_part1() {
        assert_eq!(
            Door {
                asm: EXAMPLE_ASM.into()
            }
            .part1()
            .unwrap(),
            4
        );
    }

    #[test]
    fn third_party_part1() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(Door::parse(THIRD_PARTY_INPUT)?.part1()?, 9423);
        Ok(())
    }

    #[traced_test]
    #[test]
    fn number_of_sends_before_deadlock() {
        assert_eq!(
            Door {
                asm: DEADLOCK_EXAMPLE_ASM.into()
            }
            .part2()
            .unwrap(),
            3
        );
    }

    #[test]
    fn third_party_part2() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(Door::parse(THIRD_PARTY_INPUT)?.part2()?, 7620);
        Ok(())
    }
}
