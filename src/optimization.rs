#![allow(dead_code)]

use std::fmt::Display;

use crate::{program::{Instruction, Operand, Register}, value_ids::{VidMaker, Vid}};

pub(crate) fn evaluate_instruction(vid_maker: &mut VidMaker, instr: &Instruction, left: Value, right: Value) -> Value {
    if is_instruction_no_op(instr, left, right) {
        // The instruction is a no-op, the register's value remains unchanged.
        return left;
    }

    if let (Value::Exact(_, 0), Instruction::Add(..)) = (left, instr) {
        // Adding the right-hand value to a register whose value is zero.
        // The register's result is the same value as the right-hand value.
        return right;
    }

    if let (Value::Exact(_, 1), Instruction::Mul(..)) = (left, instr) {
        // Multiplying a register whose value is one by the right-hand value.
        // The register's result is the same value as the right-hand value.
        return right;
    }

    if let (Value::Exact(_, left), Value::Exact(_, right)) = (left, right) {
        // Both values for this instruction are known exactly.
        // We can compute the result exactly as well.
        let exact_value = match instr {
            Instruction::Input(..) => unreachable!(), // not supported here
            Instruction::Add(..) => left + right,
            Instruction::Mul(..) => left * right,
            Instruction::Div(..) => left / right,
            Instruction::Mod(..) => left % right,
            Instruction::Equal(..) => {
                if left == right {
                    1
                } else {
                    0
                }
            }
        };
        return Value::Exact(vid_maker.make_new_vid(), exact_value);
    }

    if left == right {
        match instr {
            Instruction::Div(..) => {
                // Dividing two equal values, so the result is always 1.
                // The value may not be 0 since that's against the spec
                // and is therefore undefined behavior.
                return Value::Exact(vid_maker.make_new_vid(), 1);
            },
            Instruction::Mod(..) => {
                // We are calculating the remainder when dividing a value by itself,
                // so the result is always 0. The value may not be 0 since
                // that's against the spec and is therefore undefined behavior.
                return Value::Exact(vid_maker.make_new_vid(), 0);
            }
            Instruction::Equal(..) => {
                // We are comparing a value for equality against itself.
                // This is always true, so the result is always 1.
                return Value::Exact(vid_maker.make_new_vid(), 1);
            }
            _ => {}
        }
    }

    if matches!(instr, Instruction::Mul(..))
        && (matches!(left, Value::Exact(_, 0)) || matches!(right, Value::Exact(_, 0)))
    {
        // We are multiplying by 0.
        // Even though the other input is not known, the output is always 0.
        return Value::Exact(vid_maker.make_new_vid(), 0);
    }

    // We weren't able to determine the result of this instruction.
    Value::Unknown(vid_maker.make_new_vid())
}

#[rustfmt::skip]
pub(crate) fn is_instruction_no_op(instr: &Instruction, left: Value, right: Value) -> bool {
    match (left, instr, right) {
        (    _,                  Instruction::Add(..),   Value::Exact(_, 0))
        | (  Value::Exact(_, 0), Instruction::Mul(..),   _                 )
        | (  _,                  Instruction::Mul(..),   Value::Exact(_, 1))
        | (  Value::Exact(_, 0), Instruction::Div(..),   _                 )
        | (  _,                  Instruction::Div(..),   Value::Exact(_, 1)) => true,
        (Value::Exact(_, a), Instruction::Mod(..), Value::Exact(_, b)) => a < b,
        (Value::Exact(_, a), Instruction::Equal(..), Value::Exact(_, b)) => {
            // We're considering "eql a b" and storing the result in a.
            // If a == b, then a becomes 1. This is a no-op if a == b == 1.
            // If a != b, then a becomes 0. This is a no-op if a != b and a == 0.
            let sides_equal = a == b;
            (sides_equal && a == 1) || (!sides_equal && a == 0)
        }
        _ => false, // All the other cases are not no-ops.
    }
}

pub fn constant_propagation(vid_maker: &mut VidMaker, starting_registers: &[Value; 4], instructions: &[Instruction]) -> Vec<[Value; 4]> {
    let mut registers_after_instr = vec![];
    let mut next_input_id = 0;

    let mut registers = *starting_registers;
    for instr in instructions {
        if let Instruction::Input(Register(index)) = instr {
            registers[*index] = Value::Input(vid_maker.make_new_vid(), next_input_id);
            next_input_id += 1;
        } else {
            let register_index = instr.destination();
            let left = registers[register_index];
            let right = match instr.operand().unwrap() {
                Operand::Literal(lit) => Value::Exact(vid_maker.make_new_vid(), lit),
                Operand::Register(Register(r)) => registers[r],
            };

            let new_register_value = evaluate_instruction(vid_maker, instr, left, right);
            registers[register_index] = new_register_value;
        }

        registers_after_instr.push(registers);
    }

    registers_after_instr
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord)]
pub enum Value {
    Exact(Vid, i64),
    Input(Vid, usize),
    Unknown(Vid),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Self::Exact(_, left_val), &Self::Exact(_, right_val)) => left_val == right_val,
            (&l, &r) => l.vid() == r.vid(),
        }
    }
}

impl Eq for Value {}

impl Value {
    pub fn vid(&self) -> Vid {
        match self {
            Value::Exact(vid, _) | Value::Input(vid, _) | Value::Unknown(vid) => *vid,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Exact(v, val) => {
                write!(f, "{}: Exact({})", v.0, *val)
            }
            Value::Input(v, inp) => {
                write!(f, "{}: Input_{}", v.0, *inp)
            }
            Value::Unknown(v) => {
                write!(f, "{}: Unknown", v.0)
            }
        }
    }
}

pub struct Program {
    pub instructions: Vec<Instruction>,
    pub starting_registers: [Value; 4],
    pub registers_after_instr: Vec<[Value; 4]>,
    vid_maker: VidMaker,
}

impl Program {
    pub fn new(instructions: Vec<Instruction>) -> Self {
        let (mut vid_maker, starting_registers) = VidMaker::initial_registers_and_vid_maker();

        let registers_after_instr = constant_propagation(&mut vid_maker, &starting_registers, &instructions);

        Self {
            instructions,
            starting_registers,
            registers_after_instr,
            vid_maker,
        }
    }

    pub fn optimize(&self) -> Vec<Instruction> {
        let mut registers = &self.starting_registers;
        let mut new_program = vec![];

        for (index, instruction) in self.instructions.iter().enumerate() {
            let next_registers = &self.registers_after_instr[index];
            let is_no_op = registers == next_registers;

            if !is_no_op {
                new_program.push(*instruction);
            }

            registers = next_registers;
        }

        new_program
    }
}
