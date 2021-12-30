use crate::program::{Instruction, Operand};

pub fn remove_div_by_1(instructions: Vec<Instruction>) -> Vec<Instruction> {
    let mut result: Vec<Instruction> = vec![];
    for instr in instructions {
        match instr {
            Instruction::Div(_, Operand::Literal(1)) => {
                // We found a division-by-1!
                // Don't add it to the result vector, we're skipping it.
            }
            _ => {
                result.push(instr);
            }
        }
    }

    result
}
