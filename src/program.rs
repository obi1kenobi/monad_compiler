#![allow(dead_code)]

use std::{
    collections::{btree_map::Range, BTreeMap},
    fmt::Display,
};

use crate::{
    unique_ids::UniqueIdMaker,
    values::{Value, Vid},
};

/// Our programs always use i64 numbers, and this type alias
/// will let us not have to repeat that everywhere.
type RangeInclusive = std::ops::RangeInclusive<i64>;

/// The range of values that we know exactly nothing about.
pub const FULLY_UNKNOWN_RANGE: RangeInclusive = i64::MIN..=i64::MAX;

#[derive(Debug)]
pub struct Program {
    vid_maker: UniqueIdMaker<Vid>,
    initial_registers: [Value; 4],
    next_input_id: usize,

    /// The possible ranges of numbers each program value might take.
    value_ranges: BTreeMap<Vid, RangeInclusive>,
}

impl Program {
    pub fn new() -> Self {
        let mut vid_maker = Vid::unique_id_maker();
        let initial_registers = [
            Value::Exact(vid_maker.make_new_id(), 0),
            Value::Exact(vid_maker.make_new_id(), 0),
            Value::Exact(vid_maker.make_new_id(), 0),
            Value::Exact(vid_maker.make_new_id(), 0),
        ];
        let next_input_id = 0;

        // The initial values are known to be exactly 0, as initialized above.
        let mut value_ranges = BTreeMap::new();
        for register in &initial_registers {
            value_ranges.insert(register.vid(), 0..=0);
        }

        Self {
            vid_maker,
            initial_registers,
            next_input_id,
            value_ranges,
        }
    }

    pub fn initial_registers(&self) -> [Value; 4] {
        self.initial_registers
    }

    pub fn new_exact_value(&mut self, exactly: i64) -> Value {
        let vid = self.vid_maker.make_new_id();
        self.value_ranges.insert(vid, exactly..=exactly);
        Value::Exact(vid, exactly)
    }

    pub fn new_unknown_value(&mut self, range: RangeInclusive) -> Value {
        let vid = self.vid_maker.make_new_id();
        self.value_ranges.insert(vid, range);
        Value::Unknown(vid)
    }

    pub fn new_input_value(&mut self) -> Value {
        let next_input_id = self.next_input_id;
        self.next_input_id += 1;

        let vid = self.vid_maker.make_new_id();
        self.value_ranges.insert(vid, 0..=9);
        Value::Input(vid, next_input_id)
    }

    pub fn value_range(&self, vid: &Vid) -> RangeInclusive {
        self.value_ranges[vid].clone()
    }
}

/// A register in a MONAD instruction.
/// Registers w, x, y, z are Register(0) through Register(3), respectively.
#[derive(Debug, Clone, Copy)]
pub struct Register(pub usize);

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let letter = match self.0 {
            0 => "w",
            1 => "x",
            2 => "y",
            3 => "z",
            _ => unreachable!("{:?}", self),
        };

        write!(f, "{}", letter)
    }
}

/// The second operand of a MONAD instruction.
/// Can be a literal number like the `2` in `add x 2`,
/// or a register like the `y` in `add x y`.
#[derive(Debug, Clone, Copy)]
pub enum Operand {
    Literal(i64),
    Register(Register),
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Literal(l) => write!(f, "{}", l),
            Operand::Register(r) => write!(f, "{}", *r),
        }
    }
}

/// An instruction in the MONAD language.
/// See Advent of Code 2021 Day 24 for the spec: https://adventofcode.com/2021/day/24
#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    Input(Register),          // e.g. inp x
    Add(Register, Operand),   // e.g. add x 2
    Mul(Register, Operand),   // e.g. mul x 0
    Div(Register, Operand),   // e.g. div x 10
    Mod(Register, Operand),   // e.g. mod x 31
    Equal(Register, Operand), // e.g. eql x y
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Input(r) => write!(f, "inp {}", *r),
            Instruction::Add(r, o) => write!(f, "add {} {}", *r, *o),
            Instruction::Mul(r, o) => write!(f, "mul {} {}", *r, *o),
            Instruction::Div(r, o) => write!(f, "div {} {}", *r, *o),
            Instruction::Mod(r, o) => write!(f, "mod {} {}", *r, *o),
            Instruction::Equal(r, o) => write!(f, "eql {} {}", *r, *o),
        }
    }
}

impl Instruction {
    #[inline]
    pub fn destination(&self) -> usize {
        match self {
            Instruction::Input(r) => r,
            Instruction::Add(r, _) => r,
            Instruction::Mul(r, _) => r,
            Instruction::Div(r, _) => r,
            Instruction::Mod(r, _) => r,
            Instruction::Equal(r, _) => r,
        }
        .0
    }

    #[inline]
    pub fn operand(&self) -> Option<Operand> {
        match self {
            Instruction::Input(_) => None,
            Instruction::Add(_, o) => Some(*o),
            Instruction::Mul(_, o) => Some(*o),
            Instruction::Div(_, o) => Some(*o),
            Instruction::Mod(_, o) => Some(*o),
            Instruction::Equal(_, o) => Some(*o),
        }
    }
}

/// We can't impl `Display` for `&[Instruction]`, so we have to make a newtype for it.
pub struct InstructionStream<'a>(pub &'a [Instruction]);

impl<'a> Display for InstructionStream<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for instr in self.0.iter() {
            writeln!(f, "{}", instr)?;
        }
        Ok(())
    }
}

impl<'a> From<&'a [Instruction]> for InstructionStream<'a> {
    fn from(x: &'a [Instruction]) -> Self {
        Self(x)
    }
}
