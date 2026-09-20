use std::io;

use cunny_lang::{Instruction, Machine, compile};

const MOVE_RIGHT: &str = "😭😭💢";
const MOVE_LEFT: &str = "😭😭😭";
const INCREMENT: &str = "😭💢💢";
const DECREMENT: &str = "😭💢😭";
const INPUT: &str = "💢😭💢";
const OUTPUT: &str = "💢😭😭";
const LOOP_START: &str = "💢💢💢";
const LOOP_END: &str = "💢💢😭";

#[test]
fn decodes_all_eight_instructions() {
    let source = [
        MOVE_RIGHT, MOVE_LEFT, INCREMENT, DECREMENT, INPUT, OUTPUT, LOOP_START, LOOP_END,
    ]
    .join(" ");

    assert_eq!(
        compile(&source).unwrap(),
        vec![
            Instruction::MoveRight,
            Instruction::MoveLeft,
            Instruction::Increment,
            Instruction::Decrement,
            Instruction::Input,
            Instruction::Output,
            Instruction::LoopStart,
            Instruction::LoopEnd,
        ]
    );
}

#[test]
fn rejects_invalid_source() {
    assert!(compile("😭💢x").unwrap_err().contains("invalid character"));
    assert!(compile("😭💢").unwrap_err().contains("groups of three"));
}

#[test]
fn executes_a_loop() {
    let mut source = INCREMENT.repeat(8);
    source.push_str(LOOP_START);
    source.push_str(MOVE_RIGHT);
    source.push_str(&INCREMENT.repeat(8));
    source.push_str(MOVE_LEFT);
    source.push_str(DECREMENT);
    source.push_str(LOOP_END);
    source.push_str(MOVE_RIGHT);
    source.push_str(INCREMENT);
    source.push_str(OUTPUT);
    let instructions = compile(&source).unwrap();
    let mut machine = Machine::new();
    let mut input = io::empty();
    let mut output = Vec::new();

    machine.run(&instructions, &mut input, &mut output).unwrap();

    assert_eq!(output, b"A");
}

#[test]
fn reads_and_echoes_one_byte() {
    let instructions = compile(&format!("{INPUT} {OUTPUT}")).unwrap();
    let mut machine = Machine::new();
    let mut input = &b"Z"[..];
    let mut output = Vec::new();

    machine.run(&instructions, &mut input, &mut output).unwrap();

    assert_eq!(output, b"Z");
}

#[test]
fn imouto_debugger_traces_execution() {
    let instructions = compile(INCREMENT).unwrap();
    let mut machine = Machine::new();
    let mut input = io::empty();
    let mut output = Vec::new();
    let mut trace = Vec::new();

    machine
        .run_with_imouto(&instructions, &mut input, &mut output, &mut trace)
        .unwrap();

    let trace = String::from_utf8(trace).unwrap();
    assert!(trace.contains("imouto> #0001 Increment"));
    assert!(trace.contains("imouto> finished | pointer=0 cell=1 tape=[1]"));
}

#[test]
fn rejects_unmatched_loops_and_left_edge_movement() {
    let unmatched = compile(LOOP_START).unwrap();
    let move_left = compile(MOVE_LEFT).unwrap();
    let mut machine = Machine::new();
    let mut input = io::empty();
    let mut output = Vec::new();

    assert!(
        machine
            .run(&unmatched, &mut input, &mut output)
            .unwrap_err()
            .contains("has no end")
    );
    assert!(
        machine
            .run(&move_left, &mut input, &mut output)
            .unwrap_err()
            .contains("cannot move left")
    );
}
