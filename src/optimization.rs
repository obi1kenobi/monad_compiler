#![allow(dead_code)]

use crate::program::{Instruction, Operand, Register};

fn evaluate_instruction(instr: Instruction, left: Value, right: Value) -> Value {
    if let (Value::Exact(left), Value::Exact(right)) = (left, right) {
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
        return Value::Exact(exact_value);
    }

    if matches!(instr, Instruction::Mul(..))
        && (matches!(left, Value::Exact(0)) || matches!(right, Value::Exact(0)))
    {
        // We are multiplying by 0.
        // Even though the other input is not known, the output is always 0.
        return Value::Exact(0);
    }

    // We weren't able to determine the result of this instruction.
    Value::Unknown
}

#[rustfmt::skip]
fn is_instruction_no_op(instr: Instruction, left: Value, right: Value) -> bool {
    match (left, instr, right) {
        (    _,               Instruction::Add(..),   Value::Exact(0))
        | (  Value::Exact(0), Instruction::Mul(..),   _              )
        | (  _,               Instruction::Mul(..),   Value::Exact(1))
        | (  Value::Exact(0), Instruction::Div(..),   _              )
        | (  _,               Instruction::Div(..),   Value::Exact(1)) => true,
        (Value::Exact(a), Instruction::Mod(..), Value::Exact(b)) => a < b,
        (Value::Exact(a), Instruction::Equal(..), Value::Exact(b)) => {
            // We're considering "eql a b" and storing the result in a.
            // If a == b, then a becomes 1. This is a no-op if a == b == 1.
            // If a != b, then a becomes 0. This is a no-op if a != b and a == 0.
            let sides_equal = a == b;
            (sides_equal && a == 1) || (!sides_equal && a == 0)
        }
        _ => false, // All the other cases are not no-ops.
    }
}

pub fn constant_propagation(instructions: Vec<Instruction>) -> Vec<Instruction> {
    let mut result: Vec<Instruction> = vec![];
    let mut registers: [Value; 4] = [Value::Exact(0); 4];
    let mut seen_inputs = 0;
    for instr in instructions {
        let mut is_no_op = false;

        if let Instruction::Input(Register(index)) = instr {
            registers[index] = Value::Input(seen_inputs);
            seen_inputs += 1;
        } else {
            let register_index = instr.destination();
            let left = registers[register_index];
            let right = match instr.operand().unwrap() {
                Operand::Literal(lit) => Value::Exact(lit),
                Operand::Register(Register(r)) => registers[r],
            };

            let result = evaluate_instruction(instr, left, right);
            registers[register_index] = result;

            is_no_op = is_instruction_no_op(instr, left, right);
        }

        if !is_no_op {
            result.push(instr);
        }
    }

    result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    Exact(i64),
    Input(usize),
    Unknown,
}
