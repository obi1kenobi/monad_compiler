#![allow(dead_code)]

use crate::{
    program::{Instruction, Operand, Program, Register},
    values::Value,
};

pub(crate) fn evaluate_instruction(
    program: &mut Program,
    instr: Instruction,
    left: Value,
    right: Value,
) -> Value {
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
        return program.new_exact_value(exact_value);
    }

    if matches!(instr, Instruction::Mul(..))
        && (matches!(left, Value::Exact(0)) || matches!(right, Value::Exact(0)))
    {
        // We are multiplying by 0.
        // Even though the other input is not known, the output is always 0.
        return program.new_exact_value(0);
    }

    // We weren't able to determine the result of this instruction.
    program.new_unknown_value()
}

#[rustfmt::skip]
pub(crate) fn is_instruction_no_op(instr: Instruction, left: Value, right: Value) -> bool {
    // Import the variant names directly to improve readability.
    use Instruction::{Add, Mul, Div, Mod, Equal};

    match (left, instr, right) {
        (    _,               Add(..),   Value::Exact(0))       // _ + 0
        | (  Value::Exact(0), Mul(..),   _              )       // 0 * _
        | (  _,               Mul(..),   Value::Exact(1))       // _ * 1
        | (  Value::Exact(0), Div(..),   _              )       // 0 / _
        | (  _,               Div(..),   Value::Exact(1)) => {  // _ / 1
            // All these cases are always no-ops, regardless of
            // the other value in the operation.
            true
        }
        (Value::Exact(a), Mod(..), Value::Exact(b)) => {
            // a mod b computes the remainder of a when dividing by b.
            // When a < b, the remainder is a itself, which is a no-op.
            a < b
        }
        (Value::Exact(a), Equal(..), Value::Exact(b)) => {
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
    let mut program = Program::new();
    let mut new_instructions: Vec<Instruction> = vec![];

    let mut registers: [Value; 4] = program.initial_registers();

    for instr in instructions {
        if let Instruction::Input(Register(index)) = instr {
            registers[index] = program.new_input_value();
        } else {
            let register_index = instr.destination();
            let left = registers[register_index];
            let right = match instr.operand().unwrap() {
                Operand::Literal(lit) => program.new_exact_value(lit),
                Operand::Register(Register(r)) => registers[r],
            };

            let new_register_value = evaluate_instruction(&mut program, instr, left, right);
            registers[register_index] = new_register_value;

            if is_instruction_no_op(instr, left, right) {
                // This instruction is a no-op,
                // so don't include it in the new program.
                continue;
            }
        }

        new_instructions.push(instr);
    }

    new_instructions
}
