#![allow(unused_imports)]

use std::{env, fs};

use itertools::Itertools;

use crate::{
    parser::parse_program,
    program::{Instruction, InstructionStream},
};

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
        _ => unreachable!("{}", part),
    }
}

fn get_improvement_percent(original_length: usize, optimized_length: usize) -> f64 {
    let original = f64::from(original_length as i32);
    let optimized = f64::from(optimized_length as i32);

    ((original / optimized) - 1.0) * 100.0
}

fn analyze_program(input_program: Vec<Instruction>) -> Vec<Instruction> {
    let original_program = input_program.clone();

    let optimized_program = input_program;

    let original_length = original_program.len();
    let optimized_length = optimized_program.len();

    println!(
        "Original vs optimized length:    {} vs {} (-{})",
        original_length, optimized_length, original_length - optimized_length,
    );
    println!(
        "Optimized is more efficient by:  {:.2}%",
        get_improvement_percent(original_length, optimized_length)
    );

    optimized_program
}
