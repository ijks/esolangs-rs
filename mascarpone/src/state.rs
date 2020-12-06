use std::{
    collections::VecDeque,
    io::{self, BufRead, BufReader, Read, Write},
};

use crate::{
    interpreter::Interpreter, operation::Operation, stack::Stack, Error, Result, Symbol,
    STRING_LEFT_DELIM, STRING_RIGHT_DELIM,
};

#[derive(Debug)]
pub struct State<IO> {
    stack: Stack<Element>,
    pub interpreter: Interpreter,
    io: IO,
}

#[derive(Debug, Clone)]
pub enum Element {
    Symbol(Symbol),
    Operation(Operation),
    Interpreter(Option<Interpreter>),
}

impl<IO> State<IO> {
    pub fn new(io: IO) -> Self {
        Self {
            stack: Stack::new(),
            interpreter: Interpreter::default(),
            io,
        }
    }

    pub fn execute(&mut self, program: &[Symbol]) -> Result<()>
    where
        IO: Read + Write,
    {
        for &sym in program {
            self.interpreter.clone().interpret(sym, self)?
        }

        Ok(())
    }

    pub fn pop_element(&mut self) -> Result<Element> {
        self.stack.pop().ok_or(Error::EmptyStack)
    }

    pub fn pop_interpreter(&mut self) -> Result<Interpreter> {
        self.pop_interpreter_nullable()
            .and_then(|i| i.ok_or(Error::NullInterpreter))
    }

    pub fn pop_interpreter_nullable(&mut self) -> Result<Option<Interpreter>> {
        if let Element::Interpreter(i) = self.pop_element()? {
            Ok(i)
        } else {
            Err(Error::WrongElementType)
        }
    }

    pub fn pop_operation(&mut self) -> Result<Operation> {
        if let Element::Operation(o) = self.pop_element()? {
            Ok(o)
        } else {
            Err(Error::WrongElementType)
        }
    }

    pub fn pop_symbol(&mut self) -> Result<Symbol> {
        if let Element::Symbol(s) = self.pop_element()? {
            Ok(s)
        } else {
            Err(Error::WrongElementType)
        }
    }

    pub fn pop_string(&mut self) -> Result<Vec<Symbol>> {
        let mut nesting = 0u32;
        let mut string = VecDeque::new();

        if self.pop_symbol()? != STRING_RIGHT_DELIM {
            return Err(Error::MalformedString);
        }

        loop {
            let sym = self.pop_symbol()?;
            match sym {
                STRING_RIGHT_DELIM => nesting += 1,
                STRING_LEFT_DELIM => {
                    if nesting == 0 {
                        return Ok(string.into());
                    } else {
                        nesting -= 1;
                    }
                }
                _ => (),
            }

            string.push_front(sym);
        }
    }

    pub fn push_element(&mut self, elem: Element) {
        self.stack.push(elem)
    }

    pub fn push_string(&mut self, symbols: impl IntoIterator<Item = Symbol>) {
        self.push_element(Element::Symbol('['));
        for sym in symbols {
            self.push_element(Element::Symbol(sym));
        }
        self.push_element(Element::Symbol(']'));
    }

    pub fn peek_element(&self) -> Result<&Element> {
        self.stack.peek().ok_or(Error::EmptyStack)
    }

    pub fn start_quote_string(&mut self) {
        let old_interp = std::mem::replace(&mut self.interpreter, Interpreter::quote_string());
        self.interpreter.set_parent(Some(old_interp));
    }

    pub fn start_quote_symbol(&mut self) {
        let old_interp = std::mem::replace(&mut self.interpreter, Interpreter::quote_symbol());
        self.interpreter.set_parent(Some(old_interp));
    }

    pub fn read_symbol(&mut self) -> io::Result<Symbol>
    where
        IO: Read,
    {
        use utf8_chars::BufReadCharsExt;

        let mut buf_io = BufReader::with_capacity(4, &mut self.io);
        buf_io
            .read_char()?
            .ok_or(io::ErrorKind::UnexpectedEof.into())
    }

    pub fn write_symbol(&mut self, sym: Symbol) -> io::Result<()>
    where
        IO: Write,
    {
        write!(self.io, "{}", sym)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn delimiterless_string() -> impl Strategy<Value = Vec<Symbol>> {
        any::<Vec<Symbol>>().prop_filter("symbol strings must not contain delimiters", |s| {
            !(s.contains(&STRING_LEFT_DELIM) || s.contains(&STRING_RIGHT_DELIM))
        })
    }

    #[test]
    fn pop_string_fails_on_empty_stack() {
        let mut state = State::new(());

        assert!(state.pop_string().is_err());
    }

    #[test]
    fn pop_string_fails_on_delimiterless() {
        let mut state = State::new(());
        state.push_element(Element::Symbol('a'));

        assert!(state.pop_string().is_err());
    }

    proptest! {
        #[test]
        fn push_string_pop_string_succeeds(string in delimiterless_string()) {
            let mut state = State::new(());

            state.push_string(string.clone());
            state.pop_string()?;
        }

        #[test]
        fn push_string_pop_string_roundtrips(string in delimiterless_string()) {
            let mut state = State::new(());

            state.push_string(string.clone());
            let result = state.pop_string()?;

            prop_assert_eq!(result, string);
        }
    }
}
