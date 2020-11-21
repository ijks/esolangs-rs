#![feature(never_type)]

mod interpreter;
mod io;
mod queue;
mod stack;
mod state;

use std::slice;

use interpreter::Interpreter;
use io::{StringIO, SymbolIO};
use state::State;

pub type Symbol = u8;
pub type Program<'a> = slice::Iter<'a, Symbol>;

pub fn run_with_io<IO: SymbolIO>(io: IO, program: &[Symbol]) -> Result<State<IO>, String> {
    let mut state = State::new(Interpreter::default(), io);
    state.run(&mut program.iter())?;
    Ok(state)
}

pub fn run_with_input(program: &[Symbol], input: &[Symbol]) -> Result<Vec<Symbol>, String> {
    let state = run_with_io(StringIO::new(input), program)?;
    Ok(state.io.into_output())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_with_input_hello_world() -> Result<(), String> {
        let program = b"#0#10#33#100#108#114#111#119#32#44#111#108#108#101#72...............";
        let output = run_with_input(program, b"")?;
        assert_eq!(output, b"Hello, world!\n\0");
        Ok(())
    }

    #[test]
    fn run_with_input_hello_world_fancy() -> Result<(), String> {
        let program = b";#58#126#63#36!;#46#36#!;#0#1!;#0#2!;#0#3!;#0#4!;#0#5!;#0#6!;#0#7!#0#33#100#108#114#111#119#32#44#111#108#108#101#72$";
        let output = run_with_input(program, b"")?;
        assert_eq!(output, b"Hello, world!\n\0");
        Ok(())
    }

    // #[test]
    // fn run_with_input_cat_empty() -> Result<(), String> {
    //     let program = b";#44#46#35#52#50#63#42!*";
    //     let output = run_with_input(program, b"")?;
    //     assert_eq!(output, b"");
    //     Ok(())
    // }

    // #[test]
    // fn run_with_input_cat_single_line() -> Result<(), String> {
    //     let program = b";#44#46#35#52#50#63#42!*";
    //     let output = run_with_input(program, b"111\n")?;
    //     assert_eq!(output, b"111\n");
    //     Ok(())
    // }
}

//     // parity test (i.e. odd vs even)
//     let program = "#59#94#118#58!#59#35#54#57#46#!#59#35#55#57#46#128!#59#58#43#58#43#58#43#58#43#58#43#58#43#58#43#109!,m?";

//     run_emmental(StandardIO, program)
// }
