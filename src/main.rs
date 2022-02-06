#![allow(unused_imports)]

use std::{env, fs};

use itertools::Itertools;

use crate::{parser::parse_program, program::{Instruction, InstructionStream}};

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
            let analysis = analyze_program(input_program);
            println!("{:?}", analysis);
        }
        _ => unreachable!("{}", part),
    }
}

fn analyze_program(input_program: Vec<Instruction>) -> Vec<Instruction> {
    input_program
}
