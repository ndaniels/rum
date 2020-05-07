use std::convert::TryInto;
use std::fs::File;
use std::io::prelude::*;
use std::io::{stdin, stdout};
use std::process;

use crate::memory;

pub fn run(program: Vec<u32>) -> () {
    // Takes an in-memory executable image
    // as specified by the UM spec, and executes it
    // It is a c.r.e. if an instruction word has
    // an invalid opcode (14 or 15).
    let mut segmap = memory::Memory::new(program);
    // next, start calling decode() on each instruction
    // and dispatch it!
    let mut r = Registers::new();
    let mut pc = 0_usize;
    loop {
        let instr = match Instruction::decode(segmap.get_instruction(pc)) {
            Some(instr) => instr,
            None => panic!("illegal instruction"),
        };
        let op = instr.opcode;
        pc += 1;
        match op {
            Opcode::CMov => {
                if r[instr.rc] != 0 {
                    r[instr.ra] = r[instr.rb]
                }
            }
            Opcode::Load => {
                r[instr.ra] = segmap.load(
                    r[instr.rb].try_into().unwrap(),
                    r[instr.rc].try_into().unwrap(),
                );
            }
            Opcode::Store => {
                segmap.store(
                    r[instr.ra].try_into().unwrap(),
                    r[instr.rb].try_into().unwrap(),
                    r[instr.rc].try_into().unwrap(),
                );
            }
            Opcode::Add => {
                r[instr.ra] = r[instr.rb] + r[instr.rc];
            }
            Opcode::Mul => {
                r[instr.ra] = r[instr.rb] * r[instr.rc];
            }
            Opcode::Div => {
                r[instr.ra] = r[instr.rb] / r[instr.rc];
            }
            Opcode::Nand => {
                r[instr.ra] = !(r[instr.rb] & r[instr.rc]);
            }
            Opcode::Halt => {
                process::exit(0);
            }
            Opcode::MapSegment => {
                r[instr.rb] = segmap
                    .allocate(r[instr.rc].try_into().unwrap())
                    .try_into()
                    .unwrap();
            }
            Opcode::UnmapSegment => {
                segmap.deallocate(r[instr.rc].try_into().unwrap());
            }
            Opcode::Output => {
                let value = r[instr.rc] as u8;
                stdout().write(&[value]).unwrap();
            }
            Opcode::Input => match stdin().bytes().next().unwrap() {
                Ok(value) => {
                    r[instr.rc] = value as u32;
                }
                Err(e) => panic!("Bad input: {}", e),
            },
            Opcode::LoadProgram => {
                if r[instr.rb] != 0 {
                    segmap.load_segment(r[instr.rb].try_into().unwrap());
                }
                pc = r[instr.rc].try_into().unwrap();
            }
            Opcode::LoadValue => {
                r[instr.ra] = instr.value;
            }
        }
    }
}

#[derive(Debug, PartialEq)]
enum Opcode {
    CMov,
    Load,
    Store,
    Add,
    Mul,
    Div,
    Nand,
    Halt,
    MapSegment,
    UnmapSegment,
    Output,
    Input,
    LoadProgram,
    LoadValue,
}

pub fn boot(filename: &str) -> Vec<u32> {
    // Load a UM binary meeting the specification, and load it as a
    // sequence of 32-bit words in memory.
    let mut f = File::open(filename)
        .unwrap_or_else(|_| panic!("File not found: {}", filename));
    let mut contents = Vec::new();
    let mut program = Vec::new();

    match f.read_to_end(&mut contents) {
        Ok(bytes) => {
            println!("read {} bytes from {}", bytes, filename);
            // thanks to jgrillo for the following
            for i in 0..contents.len() / 4 {
                let idx = i * 4;
                let buf = &contents[idx..idx + 4];
                program.push(u32::from_be_bytes(buf.try_into().unwrap()));
            }

            program
        }
        Err(e) => {
            panic!("Encountered error while reading from {}: {}", filename, e)
        }
    }
}

// functions for instruction parsing.

fn parse_opcode(instruction: u32) -> Option<Opcode> {
    Some(match (instruction >> 28) & 0b1111 {
        0 => Opcode::CMov,
        1 => Opcode::Load,
        2 => Opcode::Store,
        3 => Opcode::Add,
        4 => Opcode::Mul,
        5 => Opcode::Div,
        6 => Opcode::Nand,
        7 => Opcode::Halt,
        8 => Opcode::MapSegment,
        9 => Opcode::UnmapSegment,
        10 => Opcode::Output,
        11 => Opcode::Input,
        12 => Opcode::LoadProgram,
        13 => Opcode::LoadValue,
        _ => return None,
    })
}

#[derive(Debug)]
struct Instruction {
    opcode: Opcode,
    ra: usize,
    rb: usize,
    rc: usize,
    value: u32,
}

impl Instruction {
    fn decode(instruction: u32) -> Option<Instruction> {
        let opcode = parse_opcode(instruction)?;
        let mut inst = Instruction { opcode, ra: 0, rb: 0, rc: 0, value: 0 };
        match inst.opcode {
            Opcode::LoadValue => {
                inst.ra = ((instruction >> 25) & 0x7) as usize;
                inst.value = ((instruction << 7) >> 7) as u32;
            }
            _ => {
                inst.ra = ((instruction >> 6) & 0x7) as usize;
                inst.rb = ((instruction >> 3) & 0x7) as usize;
                inst.rc = (instruction & 0x7) as usize;
            }
        }
        Some(inst)
    }
}

// A wrapper for encapsulating register logic. Makes it easier to experiment
// with indexing (e.g., unchecked indexing).
#[derive(Debug)]
struct Registers([u32; 8]);

impl Registers {
    pub fn new() -> Registers {
        Registers([0; 8])
    }
}

impl std::ops::Index<usize> for Registers {
    type Output = u32;

    fn index(&self, i: usize) -> &u32 {
        &self.0[i]
        // Actually does not seem to improve midmark.
        // This would probably be unsound anyway without verifying that
        // every index is correct. In practice, Instruction.{ra,rb,rc} is
        // always in the range [0, 8), so it's OK for this program.
        // unsafe { self.0.get_unchecked(i) }
    }
}

impl std::ops::IndexMut<usize> for Registers {
    fn index_mut(&mut self, i: usize) -> &mut u32 {
        &mut self.0[i]
        // Actually does not seem to improve midmark.
        // unsafe { self.0.get_unchecked_mut(i) }
    }
}
