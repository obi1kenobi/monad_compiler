#![allow(unused_imports)]

use std::{env, fs};

use itertools::Itertools;

use crate::{
    optimization::{constant_propagation, evaluate_instruction, is_instruction_no_op, Value},
    parser::parse_program,
    program::{Instruction, InstructionStream, Operand, Register},
};

mod optimization;
mod parser;
mod program;

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
    constant_propagation(input_program)
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
    println!("instruction                   post-instruction registers");
    println!("                         w     |     x     |     y     |     z");
    println!("--------------------------------------------------------------------");
    println!("<program start>    [  Exact(0) |  Exact(0) |  Exact(0) |  Exact(0) ]");

    let mut non_input_instr = 0;
    let mut non_input_instr_on_unknown_register = 0;
    let mut non_input_instr_without_any_exact = 0;

    let mut registers: [Value; 4] = [Value::Exact(0); 4];
    let mut seen_inputs = 0;
    for instr in input_program {
        let mut is_no_op = false;

        if let Instruction::Input(Register(index)) = instr {
            registers[index] = Value::Input(seen_inputs);
            seen_inputs += 1;
        } else {
            non_input_instr += 1;

            let register_index = instr.destination();
            let left = registers[register_index];
            let right = match instr.operand().unwrap() {
                Operand::Literal(lit) => Value::Exact(lit),
                Operand::Register(Register(r)) => registers[r],
            };

            if !matches!(left, Value::Exact(..)) || !matches!(right, Value::Exact(..)) {
                non_input_instr_on_unknown_register += 1;
            }

            if !matches!(left, Value::Exact(..)) && !matches!(right, Value::Exact(..)) {
                non_input_instr_without_any_exact += 1;
            }

            is_no_op = is_instruction_no_op(instr, left, right);
            let result = evaluate_instruction(instr, left, right);
            registers[register_index] = result;
        }
        let no_op_str = if is_no_op { " *NoOp" } else { "" };

        fn beautifully_padded_register(v: Value) -> String {
            let result = format!("{:?}", v);
            if result.len() % 2 == 0 {
                format!(" {:?}", v)
            } else {
                result
            }
        }

        println!(
            "{:18} [ {:^9} | {:^9} | {:^9} | {:^9} ]{}",
            format!("{}", instr),
            beautifully_padded_register(registers[0]),
            beautifully_padded_register(registers[1]),
            beautifully_padded_register(registers[2]),
            beautifully_padded_register(registers[3]),
            no_op_str
        );
    }

    println!("\nTotal non-input instructions: {:3}", non_input_instr);
    println!("- with 1+ non-exact value:    {:3} ({:.1}%)", non_input_instr_on_unknown_register, (non_input_instr_on_unknown_register * 100) as f64 / non_input_instr as f64);
    println!("- without any exact values:   {:3} ({:.1}%)", non_input_instr_without_any_exact, (non_input_instr_without_any_exact * 100) as f64 / non_input_instr as f64);
}
