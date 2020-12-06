use thiserror::Error;

use std::io::{self, Read, Write};

mod interpreter;
mod operation;
mod stack;
mod state;

pub type Symbol = char;

const STRING_LEFT_DELIM: Symbol = '[';
const STRING_RIGHT_DELIM: Symbol = ']';

#[derive(Error, Debug)]
pub enum Error {
    #[error("missing parent interpreter")]
    NoParent,
    #[error("attempted using a null interpreter")]
    NullInterpreter,
    #[error("unexpected empty stack")]
    EmptyStack,
    #[error("expected a different type of element")]
    WrongElementType,
    #[error("expected a different interpreter variant")]
    WrongInterpreterVariant,
    #[error("tried popping a string without a closing delimiter")]
    MalformedString,
    #[error("error while performing IO")]
    IOError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
struct InputOutputPair<I, O> {
    input: I,
    output: O,
}

impl<I: Read, O> Read for InputOutputPair<I, O> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.input.read(buf)
    }
}

impl<I, O: Write> Write for InputOutputPair<I, O> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.output.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }
}

pub fn run_with_io<IO: io::Read + io::Write>(io: IO, program: &str) -> Result<()> {
    let mut state = state::State::new(io);
    let program = program.chars().collect::<Vec<_>>();

    state.execute(program.as_slice())
}

pub fn run(program: &str) -> Result<()> {
    let io = InputOutputPair {
        input: io::stdin(),
        output: io::stdout(),
    };
    run_with_io(io, program)
}

pub fn compute(program: &str, input: &str) -> Result<String> {
    let mut io = InputOutputPair {
        input: io::Cursor::new(input),
        output: Vec::<u8>::new(),
    };
    run_with_io(&mut io, program)?;

    Ok(String::from_utf8(io.output).expect("output should always be UTF-8"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_simple_output() {
        let program = "[o[ll]eh].........";

        assert_eq!(compute(program, "").unwrap(), "]he]ll[o[");
    }
}
