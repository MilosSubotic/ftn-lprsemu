use pest::iterators::Pair;
use pest::Parser;
use std::collections::HashMap;

use crate::instructions::Instruction;
use crate::op;
use crate::processor::{RAM_SIZE, ROM_SIZE};

#[derive(Parser)]
#[grammar = "src/assembler/isa.pest"]
pub struct AsmFileParser;

type RawInstructions<'a> = Vec<RawInstruction<'a>>;
type Labels<'a> = HashMap<&'a str, usize>;

struct AsmFile<'a> {
    data: Vec<u16>,
    instructions: Vec<RawInstruction<'a>>,
    labels: HashMap<&'a str, usize>,
}

#[derive(Debug)]
enum RawInstruction<'a> {
    Label {
        op: &'a str,
        label: &'a str,
    },
    Reg2 {
        op: &'a str,
        z: &'a str,
        x: &'a str,
    },
    Reg3 {
        op: &'a str,
        z: &'a str,
        x: &'a str,
        y: &'a str,
    },
}

fn parse_instruction_l(pair: Pair<'_, Rule>) -> Result<RawInstruction, ()> {
    let mut pairs;
    if let Rule::instr_l = pair.as_rule() {
        pairs = pair.into_inner();
    } else {
        return Err(());
    }
    match (pairs.next(), pairs.next()) {
        (Some(op), Some(label)) => Ok(RawInstruction::Label {
            op: op.as_str(),
            label: label.as_str(),
        }),
        _ => Err(()),
    }
}

fn parse_instruction_2r(pair: Pair<'_, Rule>) -> Result<RawInstruction, ()> {
    let mut pairs;
    if let Rule::instr_2r = pair.as_rule() {
        pairs = pair.into_inner();
    } else {
        return Err(());
    }
    let op = pairs.next();
    if let (Some(r1), Some(r2)) = (
        pairs.next().into_iter().next(),
        pairs.next().into_iter().next(),
    ) {
        match (op, r1.into_inner().next(), r2.into_inner().next()) {
            (Some(op), Some(r1), Some(r2)) => Ok(RawInstruction::Reg2 {
                op: op.as_str(),
                z: r1.as_str(),
                x: r2.as_str(),
            }),
            _ => Err(()),
        }
    } else {
        Err(())
    }
}

fn parse_instruction_3r(pair: Pair<'_, Rule>) -> Result<RawInstruction, ()> {
    let mut pairs;
    if let Rule::instr_3r = pair.as_rule() {
        pairs = pair.into_inner();
    } else {
        return Err(());
    }
    let op = pairs.next();
    if let (Some(r1), Some(r2), Some(r3)) = (
        pairs.next().into_iter().next(),
        pairs.next().into_iter().next(),
        pairs.next().into_iter().next(),
    ) {
        match (
            op,
            r1.into_inner().next(),
            r2.into_inner().next(),
            r3.into_inner().next(),
        ) {
            (Some(op), Some(r1), Some(r2), Some(r3)) => Ok(RawInstruction::Reg3 {
                op: op.as_str(),
                z: r1.as_str(),
                x: r2.as_str(),
                y: r3.as_str(),
            }),
            _ => Err(()),
        }
    } else {
        Err(())
    }
}

impl<'a> TryFrom<Pair<'a, Rule>> for RawInstruction<'a> {
    type Error = ();
    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        if let Rule::instruction = value.as_rule() {
            if let Some(value) = value.into_inner().next() {
                match value.as_rule() {
                    Rule::instr_l => parse_instruction_l(value),
                    Rule::instr_2r => parse_instruction_2r(value),
                    Rule::instr_3r => parse_instruction_3r(value),
                    _ => Err(()),
                }
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}

fn parse_instructions<'a>(
    instructions: RawInstructions<'a>,
    labels: Labels<'a>,
) -> Result<Vec<Instruction>, ParsingError> {
    let mut processed: Vec<Instruction> = Vec::with_capacity(instructions.len());
    for ins in instructions {
        match ins {
            RawInstruction::Label { op, label } => {
                let line = match labels.get(label) {
                    Some(l) => *l as u16,
                    None => return Err(ParsingError::UndefinedLabel),
                };
                match op {
                    "jmp" => processed.push(op![jmp line]),
                    "jmpz" => processed.push(op![jmpz line]),
                    "jmps" => processed.push(op![jmps line]),
                    "jmpc" => processed.push(op![jmpc line]),
                    "jmpnz" => processed.push(op![jmpnz line]),
                    "jmpns" => processed.push(op![jmpns line]),
                    "jmpnc" => processed.push(op![jmpnc line]),
                    _ => return Err(ParsingError::InvalidInstruction),
                }
            }
            RawInstruction::Reg2 { op, z, x } => match (op, z.parse::<u8>(), x.parse::<u8>()) {
                ("mov", Ok(z), Ok(x)) => processed.push(op![mov z, x]),
                ("inc", Ok(z), Ok(x)) => processed.push(op![inc z, x]),
                ("dec", Ok(z), Ok(x)) => processed.push(op![dec z, x]),
                ("shl", Ok(z), Ok(x)) => processed.push(op![shl z, x]),
                ("shr", Ok(z), Ok(x)) => processed.push(op![shr z, x]),
                ("ashl", Ok(z), Ok(x)) => processed.push(op![ashl z, x]),
                ("ashr", Ok(z), Ok(x)) => processed.push(op![ashr z, x]),
                ("ld", Ok(z), Ok(y)) => processed.push(op![ld z, y]),
                ("st", Ok(z), Ok(x)) => processed.push(op![st z, x]),
                _ => return Err(ParsingError::InvalidInstruction),
            },
            RawInstruction::Reg3 { op, z, x, y } => {
                match (op, z.parse::<u8>(), x.parse::<u8>(), y.parse::<u8>()) {
                    ("add", Ok(z), Ok(x), Ok(y)) => processed.push(op![add z, x, y]),
                    ("sub", Ok(z), Ok(x), Ok(y)) => processed.push(op![sub z, x, y]),
                    ("and", Ok(z), Ok(x), Ok(y)) => processed.push(op![and z, x, y]),
                    ("or", Ok(z), Ok(x), Ok(y)) => processed.push(op![or z, x, y]),
                    _ => return Err(ParsingError::InvalidInstruction),
                }
            }
        }
    }
    Ok(processed)
}

#[derive(Debug)]
pub enum ParsingError {
    Filesystem(std::io::Error),
    Pest(pest::error::Error<Rule>),
    InvalidInstruction,
    UndefinedLabel,
    MalformedFile,
    UnexpectedToken,
}

impl From<std::io::Error> for ParsingError {
    fn from(e: std::io::Error) -> Self {
        Self::Filesystem(e)
    }
}

impl From<pest::error::Error<Rule>> for ParsingError {
    fn from(e: pest::error::Error<Rule>) -> Self {
        Self::Pest(e)
    }
}

