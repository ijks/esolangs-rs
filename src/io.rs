use std::{
    error::Error,
    io::{self, Read, Write},
};

use crate::Symbol;

pub trait SymbolIO {
    type Error: Error;

    fn read_symbol(&mut self) -> Result<Symbol, Self::Error>;
    fn write_symbol(&mut self, sym: Symbol) -> Result<(), Self::Error>;
}

pub struct StandardIO;

impl SymbolIO for StandardIO {
    type Error = io::Error;

    fn read_symbol(&mut self) -> Result<Symbol, Self::Error> {
        io::stdin()
            .bytes()
            .next()
            .unwrap_or(Err(io::ErrorKind::UnexpectedEof.into()))
    }

    fn write_symbol(&mut self, sym: Symbol) -> Result<(), Self::Error> {
        let mut stdout = io::stdout();
        stdout.write(&[sym]).map(|_| ())
    }
}

pub struct StringIO<'s> {
    input: &'s [Symbol],
    output: Vec<Symbol>,
}

impl<'s> StringIO<'s> {
    pub fn new(input: &'s [Symbol]) -> Self {
        Self {
            input,
            output: Vec::new(),
        }
    }

    pub fn into_output(self) -> Vec<Symbol> {
        self.output
    }
}

impl<'s> SymbolIO for StringIO<'s> {
    type Error = !;

    fn read_symbol(&mut self) -> Result<Symbol, Self::Error> {
        const EOT: Symbol = 4;

        Ok(match self.input {
            [] => EOT,
            [s, rest @ ..] => {
                self.input = rest;
                *s
            }
        })
    }

    fn write_symbol(&mut self, sym: Symbol) -> Result<(), Self::Error> {
        self.output.push(sym);
        Ok(())
    }
}
