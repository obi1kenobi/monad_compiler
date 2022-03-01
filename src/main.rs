#![allow(unused_imports)]

use std::{env, fs};

use itertools::Itertools;
use optimization::Program;

use crate::{
    optimization::{constant_propagation, evaluate_instruction, is_instruction_no_op},
    parser::parse_program,
    program::{Instruction, InstructionStream, Operand, Register},
    values::Value,
};

mod optimization;
mod parser;
mod program;
mod values;
mod annotated_instr;
mod unique_ids;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut reversed_args: Vec<_> = args.iter().map(|x| x.as_str()).rev().collect();

    reversed_args
        .pop()
        .expect("Expected the executable name to be the first argument, but was missing");

    let part = reversed_args.pop().expect("part number");
    let input_file = reversed_args.pop().expect("input file");
    let content = fs::read_to_string(input_file).unwrap();

    let input_program: Vec<Instruction> = parse_program(content.as_str());

    match part {
        "analyze" => {
            analyze_program(input_program);
        }
        "registers" => {
            simulate_registers(input_program);
        }
        _ => unreachable!("{}", part),
    }
}

fn get_improvement_percent(original_length: usize, optimized_length: usize) -> f64 {
    let original = f64::from(original_length as i32);
    let optimized = f64::from(optimized_length as i32);

    ((original / optimized) - 1.0) * 100.0
}

fn optimize_program(input_program: Vec<Instruction>) -> Vec<Instruction> {
    Program::new(input_program).optimize()
}

fn analyze_program(input_program: Vec<Instruction>) -> Vec<Instruction> {
    let original_program = input_program.clone();

    let optimized_program = optimize_program(input_program);

    let original_length = original_program.len();
    let optimized_length = optimized_program.len();

    println!(
        "Original vs optimized length:    {} vs {} (-{})",
        original_length,
        optimized_length,
        original_length - optimized_length,
    );
    println!(
        "Optimized is more efficient by:  {:.2}%",
        get_improvement_percent(original_length, optimized_length)
    );

    optimized_program
}

fn simulate_registers(input_program: Vec<Instruction>) {
    fn beautifully_padded_register(v: Value) -> String {
        let result = format!("{}", v);
        if result.len() % 2 == 0 {
            format!(" {}", v)
        } else {
            result
        }
    }

    println!("instruction           post-instruction registers");
    println!("                 w     |     x     |     y     |     z");
    println!("-----------------------------------------------------------------------------------");

    let program = Program::new(input_program);

    println!(
        "<start>    [ {:^15} | {:^15} | {:^15} | {:^15} ]",
        beautifully_padded_register(program.starting_registers[0]),
        beautifully_padded_register(program.starting_registers[1]),
        beautifully_padded_register(program.starting_registers[2]),
        beautifully_padded_register(program.starting_registers[3]),
    );

    let mut non_input_instr = 0;
    let mut non_input_instr_on_unknown_register = 0;
    let mut non_input_instr_without_any_exact = 0;

    let mut last_registers = &program.starting_registers;
    for (instr, registers) in program.instructions.iter().zip(program.registers_after_instr.iter()) {
        let is_no_op = last_registers == registers;

        if !matches!(instr, Instruction::Input(..)) {
            non_input_instr += 1;

            let register_index = instr.destination();
            let left_is_exact = matches!(last_registers[register_index], Value::Exact(..));
            let right_is_exact = match instr.operand().unwrap() {
                Operand::Literal(_) => true,
                Operand::Register(Register(r)) => matches!(last_registers[r], Value::Exact(..)),
            };

            if !left_is_exact || !right_is_exact {
                non_input_instr_on_unknown_register += 1;
            }

            if !left_is_exact && !right_is_exact {
                non_input_instr_without_any_exact += 1;
            }
        }

        let no_op_str = if is_no_op { " *NoOp" } else { "" };

        println!(
            "{:10} [ {:^15} | {:^15} | {:^15} | {:^15} ]{}",
            format!("{}", instr),
            beautifully_padded_register(registers[0]),
            beautifully_padded_register(registers[1]),
            beautifully_padded_register(registers[2]),
            beautifully_padded_register(registers[3]),
            no_op_str
        );

        last_registers = registers;
    }

    println!("\nTotal non-input instructions: {:3}", non_input_instr);
    println!("- with 1+ non-exact value:    {:3} ({:.1}%)", non_input_instr_on_unknown_register, (non_input_instr_on_unknown_register * 100) as f64 / non_input_instr as f64);
    println!("- without any exact values:   {:3} ({:.1}%)", non_input_instr_without_any_exact, (non_input_instr_without_any_exact * 100) as f64 / non_input_instr as f64);
}
