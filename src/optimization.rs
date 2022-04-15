#![allow(dead_code)]

use crate::{program::{Instruction, Operand, Register, Program}, unique_ids::UniqueIdMaker, values::{Vid, Value}};

pub(crate) fn evaluate_instruction(program: &mut Program, instr: Instruction, left: Value, right: Value) -> Value {
    match instr {
        Instruction::Input(..) => unreachable!(),
        Instruction::Add(..) => evaluate_add(program, left, right),
        Instruction::Mul(..) => evaluate_mul(program, left, right),
        Instruction::Div(..) => evaluate_div(program, left, right),
        Instruction::Mod(..) => evaluate_mod(program, left, right),
        Instruction::Equal(..) => evaluate_equal(program, left, right),
    }
}

fn evaluate_add(program: &mut Program, left: Value, right: Value) -> Value {
    match (left, right) {
        (Value::Exact(_, left), Value::Exact(_, right)) => {
            // Both values known exactly, so the output is exact too.
            program.new_exact_value(left + right)
        }
        (_, Value::Exact(_, 0)) => left,   // left + 0 = left
        (Value::Exact(_, 0), _) => right,  // 0 + right = right
        _ => program.new_unknown_value(),
    }
}

fn evaluate_mul(program: &mut Program, left: Value, right: Value) -> Value {
    match (left, right) {
        (Value::Exact(_, left), Value::Exact(_, right)) => {
            // Both values known exactly, so the output is exact too.
            program.new_exact_value(left * right)
        }
        (_, Value::Exact(_, 0)) | (Value::Exact(_, 0), _) => {
            // We are multiplying by 0.
            // No matter what the other value is, the output is always 0.
            program.new_exact_value(0)
        }
        (_, Value::Exact(_, 1)) => left,   // left * 1 = left
        (Value::Exact(_, 1), _) => right,  // 1 * right = right
        _ => program.new_unknown_value(),
    }
}

fn evaluate_div(program: &mut Program, left: Value, right: Value) -> Value {
    match (left, right) {
        (Value::Exact(_, left), Value::Exact(_, right)) => {
            // Both values known exactly, so the output is exact too.
            program.new_exact_value(left / right)
        }
        (Value::Exact(_, 0), _) => left,  // 0 / right = 0
        (_, Value::Exact(_, 1)) => left,  // left / 1 = left
        _ => program.new_unknown_value(),
    }
}

fn evaluate_mod(program: &mut Program, left: Value, right: Value) -> Value {
    match (left, right) {
        (Value::Exact(_, left), Value::Exact(_, right)) => {
            // Both values known exactly, so the output is exact too.
            program.new_exact_value(left % right)
        }
        (Value::Exact(_, 0), _) => {
            // The remainder when dividing 0 by any number is always 0.
            program.new_exact_value(0)
        }
        (_, Value::Exact(_, 1)) => {
            // Any number divided by 1 produces a remainder of 0.
            program.new_exact_value(0)
        }
        _ => program.new_unknown_value(),
    }
}

fn evaluate_equal(program: &mut Program, left: Value, right: Value) -> Value {
    if left == right {
        // The two values are equal,
        // so we know this instruction is guaranteed to produce 1.
        return program.new_exact_value(1);
    }

    // The values are *not known* to be equal: they might or might not be equal.
    // Consider e.g. Unknown(Vid(i)) and Unknown(Vid(j)) with i != j.
    // The only case where we can (currently) know for sure that the two values
    // are non-equal is if both values are Value::Exact representing different numbers.
    // Otherwise, the outcome of this instruction is unknown.
    match (left, right) {
        (Value::Exact(_, l), Value::Exact(_, r)) if l != r => {
            program.new_exact_value(0)
        }
        _ => program.new_unknown_value(),
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
            let destination_register = instr.destination();
            let left = registers[destination_register];
            let right = match instr.operand().unwrap() {
                Operand::Literal(lit) => program.new_exact_value(lit),
                Operand::Register(Register(r)) => registers[r],
            };

            let previous_register_value = registers[destination_register];

            let new_register_value = evaluate_instruction(&mut program, instr, left, right);
            registers[destination_register] = new_register_value;

            if previous_register_value == new_register_value {
                // This instruction is a no-op,
                // so omit it from the list of instructions.
                continue;
            }
        }

        new_instructions.push(instr);
    }

    new_instructions
}
