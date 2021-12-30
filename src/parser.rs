use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1, line_ending, one_of, space1},
    combinator::{map, map_res, opt, recognize},
    multi::many1,
    sequence::tuple,
    IResult,
};

use crate::program::{Register, Operand, Instruction};

fn register(input: &str) -> IResult<&str, Register> {
    let (remainder, matched_char) = one_of("wxyz")(input)?;
    let register_id = match matched_char {
        'w' => 0,
        'x' => 1,
        'y' => 2,
        'z' => 3,
        _ => unreachable!("{}", matched_char),
    };

    Ok((remainder, Register(register_id)))
}

fn text_signed_int(input: &str) -> IResult<&str, i64> {
    map_res(recognize(tuple((opt(char('-')), digit1))), |value: &str| {
        value.parse()
    })(input)
}

fn operand(input: &str) -> IResult<&str, Operand> {
    if let Ok((remainder, register)) = register(input) {
        Ok((remainder, Operand::Register(register)))
    } else {
        map(text_signed_int, Operand::Literal)(input)
    }
}

fn input_instruction(input: &str) -> IResult<&str, Instruction> {
    map(
        tuple((tag("inp"), space1, register, opt(line_ending))),
        |(_, _, reg, _)| Instruction::Input(reg),
    )(input)
}

fn binary_instruction(input: &str) -> IResult<&str, Instruction> {
    map(
        tuple((
            alt((tag("add"), tag("mul"), tag("div"), tag("mod"), tag("eql"))),
            space1,
            register,
            space1,
            operand,
            opt(line_ending),
        )),
        |(instr, _, reg, _, val, _)| match instr {
            "add" => Instruction::Add(reg, val),
            "mul" => Instruction::Mul(reg, val),
            "div" => Instruction::Div(reg, val),
            "mod" => Instruction::Mod(reg, val),
            "eql" => Instruction::Equal(reg, val),
            _ => unreachable!("{}", instr),
        },
    )(input)
}

fn instruction(input: &str) -> IResult<&str, Instruction> {
    alt((input_instruction, binary_instruction))(input)
}

pub fn parse_program(input: &str) -> Vec<Instruction> {
    let (remainder, program) = many1(instruction)(input).unwrap();
    assert!(remainder.is_empty());
    program
}
