#![allow(dead_code)]

use crate::{program::{Instruction, Operand, Register, Program}, unique_ids::UniqueIdMaker, values::{Vid, Value}};

pub(crate) fn evaluate_instruction(program: &mut Program, instr: Instruction, left: Value, right: Value) -> Value {
    // If both the left and the right value are known exactly,
    // we can always calculate an exact result.
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
        return program.new_exact_value(exact_value);
    }

    // For some instructions, we may still be able to figure out the result
    // even though one of the values is unknown. The resulting value might not be exact,
    // but may still indicate information such as "the same value as the left input value,"
    // which is useful information for downstream optimization passes.
    let maybe_value = match instr {
        Instruction::Input(..) => unreachable!(), // not supported here
        Instruction::Add(..) => {
            match (left, right) {
                (_, Value::Exact(_, 0)) => Some(left),   // left + 0 = p
                (Value::Exact(_, 0), _) => Some(right),  // 0 + right = right
                _ => None,
            }
        }
        Instruction::Mul(..) => {
            match (left, right) {
                (_, Value::Exact(_, 0)) | (Value::Exact(_, 0), _) => {
                    // We are multiplying by 0.
                    // Even though the other input is not known, the output is always 0.
                    Some(program.new_exact_value(0))
                }
                (_, Value::Exact(_, 1)) => Some(left),   // left * 1 = left
                (Value::Exact(_, 1), _) => Some(right),  // 1 * right = right
                _ => None,
            }
        }
        Instruction::Div(..) => {
            match (left, right) {
                (Value::Exact(_, 0), _) => Some(left),  // 0 / right = 0
                (_, Value::Exact(_, 1)) => Some(left),  // left / 1 = left
                _ => None,
            }
        }
        Instruction::Equal(..) => {
            if left == right {
                // Situations where both left and right are Exact(..) values were already handled
                // in the code above. However, it's possible for us to know that two values
                // are equal even if we don't know what number they represent:
                //   Unknown(Vid(i)) is known to be equal to Unknown(Vid(j)) when i == j.
                // However, when i != j, then we don't know whether Unknown(Vid(i)) and
                // Unknown(Vid(j)) are equal or not, so we can't determine anything about that case.
                Some(program.new_exact_value(1))
            } else {
                None
            }
        }
        _ => None,
    };

    if let Some(value) = maybe_value {
        value
    } else {
        // We weren't able to determine the result of this instruction, return a new Unknown value.
        program.new_unknown_value()
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

            let previous_register_value = registers[register_index];

            let new_value = evaluate_instruction(&mut program, instr, left, right);
            registers[register_index] = new_value;

            if previous_register_value == new_value {
                // This instruction is a no-op,
                // so omit it from the list of instructions.
                continue;
            }
        }

        new_instructions.push(instr);
    }

    new_instructions
}
