#![allow(dead_code)]

use crate::program::{Instruction, Operand, Register};

fn evaluate_instruction(instr: Instruction, left: Value, right: Value) -> Value {
    if let (Value::Exact(left), Value::Exact(right)) = (left, right) {
        // Both values for this instruction are known exactly.
        // We can compute the result exactly as well.
        let exact_value = match instr {
            Instruction::Input(_) => unreachable!(), // not supported here
            Instruction::Add(_, _) => left + right,
            Instruction::Mul(_, _) => left * right,
            Instruction::Div(_, _) => left / right,
            Instruction::Mod(_, _) => left % right,
            Instruction::Equal(_, _) => {
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
    match (instr, left, right) {
        (  Instruction::Add(..),               _, Value::Exact(0))
        | (Instruction::Mul(..), Value::Exact(0),               _)
        | (Instruction::Mul(..),               _, Value::Exact(1))
        | (Instruction::Div(..), Value::Exact(0),               _)
        | (Instruction::Div(..),               _, Value::Exact(1)) => true,
        (Instruction::Mod(..), Value::Exact(a), Value::Exact(b)) => a < b,
        (Instruction::Equal(..), Value::Exact(a), Value::Exact(b)) => {
            // Equal is a no-op if the sides are equal and a == 1,
            // or if they are not and a == 0.
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
