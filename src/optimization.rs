#![allow(dead_code)]

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use crate::{
    program::{Instruction, Operand, Register},
    value_ids::{Vid, VidMaker},
};

pub(crate) fn evaluate_instruction(
    vid_maker: &mut VidMaker,
    instr: &Instruction,
    left: Value,
    right: Value,
) -> Value {
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
            }
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

pub fn constant_propagation(
    vid_maker: &mut VidMaker,
    starting_registers: &[Value; 4],
    instructions: &[Instruction],
) -> Vec<[Value; 4]> {
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
    pub value_used: BTreeSet<Vid>,
    vid_maker: VidMaker,
}

impl Program {
    pub fn new(instructions: Vec<Instruction>) -> Self {
        let (mut vid_maker, starting_registers) = VidMaker::initial_registers_and_vid_maker();

        let registers_after_instr =
            constant_propagation(&mut vid_maker, &starting_registers, &instructions);

        let value_used =
            find_used_values(starting_registers, &registers_after_instr, &instructions);

        Self {
            instructions,
            starting_registers,
            registers_after_instr,
            value_used,
            vid_maker,
        }
    }

    pub fn optimize(&self) -> Vec<Instruction> {
        let mut registers = &self.starting_registers;
        let mut new_program = vec![];

        for (index, instruction) in self.instructions.iter().enumerate() {
            let next_registers = &self.registers_after_instr[index];
            let is_no_op = registers == next_registers;
            let is_used_value = self.value_used.contains(&next_registers[instruction.destination()].vid());

            // dbg!(&self.value_used);

            if !is_no_op && is_used_value {
                new_program.push(*instruction);
            } else if !is_used_value {
                println!("unused: {}: {}, value {}", index, instruction, next_registers[instruction.destination()]);
            }

            registers = next_registers;
        }

        new_program
    }
}

fn find_used_values(
    starting_registers: [Value; 4],
    registers_after_instr: &[[Value; 4]],
    instructions: &[Instruction],
) -> BTreeSet<Vid> {
    let mut used_values: BTreeSet<Vid> = Default::default();

    // The z (last) register's value at the end of the program is used,
    // by the definition of the MONAD language and the problem statement.
    used_values.insert(registers_after_instr.last().unwrap().last().unwrap().vid());

    let mut registers_before_instr = vec![&starting_registers];
    registers_before_instr.extend(registers_after_instr[..registers_after_instr.len() - 1].iter());
    assert_eq!(instructions.len(), registers_before_instr.len());

    let reversed_instructions = instructions.iter().rev();
    let reversed_registers_before = registers_before_instr.into_iter().rev();
    let reversed_registers_after = registers_after_instr.iter().rev();

    for (instr, (registers_before, registers_after)) in reversed_instructions.zip(reversed_registers_before.zip(reversed_registers_after)) {
        let source_value = registers_before[instr.destination()];
        let operand_register_value = match instr.operand() {
            Some(Operand::Register(r)) => Some(registers_before[r.0]),
            _ => None,
        };
        let operand_value = match instr.operand() {
            Some(Operand::Register(r)) => Some(registers_before[r.0]),
            Some(Operand::Literal(l)) => Some(Value::Exact(Vid::UNDEFINED, l)),
            None => None,
        };
        let destination_value = registers_after[instr.destination()];

        let special_case_mul_unused_source_values = {
            // Special case: multiplying by 0 clears any prior state from a register.
            // That register's prior value isn't used -- it doesn't matter what it is.
            //
            // Since we can prove this is a multiplication by zero regardless of where the zero is,
            // we can rewrite this to be a multiplication against literal zero and avoid the use
            // of the operand register (if any). The operand register (if any) is also not used.
            matches!(instr, Instruction::Mul(..)) && (
                matches!(source_value, Value::Exact(_, 0)) ||
                matches!(operand_value, Some(Value::Exact(_, 0)))
            )
        };
        let special_case_eql_unused_source_values = {
            // Special case: comparing two known-equal values always produces the Exact(1) value.
            // We can always rewrite this instruction as a "compare the register against itself",
            // in which form it uses neither the source register's value nor the operand register's
            // value (if any) -- all values equal themselves so the output is correct regardless
            // of the registers' prior state.
            matches!(instr, Instruction::Equal(..)) && source_value == operand_value.unwrap()
        };

        if !special_case_mul_unused_source_values && !special_case_eql_unused_source_values {
            match instr {
                Instruction::Input(..) => {
                    // The value that was previously in the register is ignored and overwritten.
                    // This does not count as a use.
                }
                Instruction::Add(..) |
                Instruction::Mul(..) |
                Instruction::Div(..) |
                Instruction::Mod(..) |
                Instruction::Equal(..) => {
                    // If this instruction is a no-op (i.e. the register's value is unchanged)
                    // then we know the instruction is going to get eliminated and we don't consider
                    // the operand register's value (if any) as used.
                    //
                    // If the register's value changes, then the instruction is not a no-op
                    // but may still be dead code: we check if the destination value is used.
                    // If the destination value was used, then the source register's value
                    // and the operand register's value (if any) are both used.
                    if destination_value != source_value && used_values.contains(&destination_value.vid()) {
                        used_values.insert(source_value.vid());

                        // If the instruction uses an operand register, and the register's value
                        // is exactly known, then there's no dependency on the operand register's
                        // value: we can rewrite the instruction to use a literal instead.
                        if let Some(operand_value) = operand_register_value {
                            if !matches!(operand_value, Value::Exact(..)) {
                                used_values.insert(operand_value.vid());
                            }
                        }
                    }
                }
            }
        }
    }

    used_values
}
