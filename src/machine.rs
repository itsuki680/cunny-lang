use std::io::{Read, Write};

use crate::language::Instruction;

#[derive(Debug, PartialEq, Eq)]
pub struct Machine {
    tape: Vec<u8>,
    pointer: usize,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            tape: vec![0],
            pointer: 0,
        }
    }

    pub fn run(
        &mut self,
        instructions: &[Instruction],
        input: &mut impl Read,
        output: &mut impl Write,
    ) -> Result<(), String> {
        self.run_inner(instructions, input, output, None)
    }

    pub fn run_with_imouto(
        &mut self,
        instructions: &[Instruction],
        input: &mut impl Read,
        output: &mut impl Write,
        trace: &mut impl Write,
    ) -> Result<(), String> {
        self.run_inner(instructions, input, output, Some(trace))
    }

    fn run_inner(
        &mut self,
        instructions: &[Instruction],
        input: &mut impl Read,
        output: &mut impl Write,
        mut trace: Option<&mut dyn Write>,
    ) -> Result<(), String> {
        let jump_table = build_jump_table(instructions)?;
        let mut instruction_pointer = 0;

        while instruction_pointer < instructions.len() {
            if let Some(debugger) = trace.as_deref_mut() {
                writeln!(
                    debugger,
                    "imouto> #{:04} {:?} | pointer={} cell={} tape={:?}",
                    instruction_pointer + 1,
                    instructions[instruction_pointer],
                    self.pointer,
                    self.tape[self.pointer],
                    self.tape
                )
                .map_err(|error| format!("could not write debugger trace: {error}"))?;
            }

            match &instructions[instruction_pointer] {
                Instruction::MoveRight => {
                    self.pointer += 1;
                    if self.pointer == self.tape.len() {
                        self.tape.push(0);
                    }
                }
                Instruction::MoveLeft => {
                    if self.pointer == 0 {
                        return Err(format!(
                            "cannot move left at instruction {}: already at the beginning of the tape",
                            instruction_pointer + 1
                        ));
                    }
                    self.pointer -= 1;
                }
                Instruction::Increment => {
                    self.tape[self.pointer] = self.tape[self.pointer].wrapping_add(1);
                }
                Instruction::Decrement => {
                    self.tape[self.pointer] = self.tape[self.pointer].wrapping_sub(1);
                }
                Instruction::Output => output
                    .write_all(&[self.tape[self.pointer]])
                    .map_err(|error| format!("could not write output: {error}"))?,
                Instruction::Input => {
                    let mut byte = [0];
                    let bytes_read = input
                        .read(&mut byte)
                        .map_err(|error| format!("could not read input: {error}"))?;
                    self.tape[self.pointer] = if bytes_read == 0 { 0 } else { byte[0] };
                }
                Instruction::LoopStart if self.tape[self.pointer] == 0 => {
                    instruction_pointer = jump_table[instruction_pointer];
                }
                Instruction::LoopEnd if self.tape[self.pointer] != 0 => {
                    instruction_pointer = jump_table[instruction_pointer];
                }
                Instruction::LoopStart | Instruction::LoopEnd => {}
            }

            instruction_pointer += 1;
        }

        if let Some(debugger) = trace {
            writeln!(
                debugger,
                "imouto> finished | pointer={} cell={} tape={:?}",
                self.pointer, self.tape[self.pointer], self.tape
            )
            .map_err(|error| format!("could not write debugger trace: {error}"))?;
        }

        Ok(())
    }
}

impl Default for Machine {
    fn default() -> Self {
        Self::new()
    }
}

fn build_jump_table(instructions: &[Instruction]) -> Result<Vec<usize>, String> {
    let mut jump_table = vec![0; instructions.len()];
    let mut stack = Vec::new();

    for (position, instruction) in instructions.iter().enumerate() {
        match instruction {
            Instruction::LoopStart => stack.push(position),
            Instruction::LoopEnd => {
                let start = stack.pop().ok_or_else(|| {
                    format!("loop end at instruction {} has no beginning", position + 1)
                })?;
                jump_table[start] = position;
                jump_table[position] = start;
            }
            _ => {}
        }
    }

    if let Some(start) = stack.last() {
        return Err(format!(
            "loop beginning at instruction {} has no end",
            start + 1
        ));
    }

    Ok(jump_table)
}
