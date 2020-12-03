use std::{collections::VecDeque, iter::Extend};

use crate::{interpreter::Interpreter, operation::Operation, stack::Stack, Error, Result, Symbol};

#[derive(Debug)]
pub struct State {
    stack: Stack<Element>,
    pub interpreter: Interpreter,
}

#[derive(Debug, Clone)]
pub enum Element {
    Symbol(Symbol),
    Operation(Operation),
    Interpreter(Interpreter),
}

impl State {
    pub fn new(interpreter: Interpreter) -> Self {
        Self {
            stack: Stack::new(),
            interpreter,
        }
    }

    pub fn pop_element(&mut self) -> Result<Element> {
        self.stack.pop().ok_or(Error::EmptyStack)
    }

    pub fn pop_interpreter(&mut self) -> Result<Interpreter> {
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

        if self.pop_symbol()? != ']' {
            return Err(Error::MalformedString);
        }

        loop {
            let sym = self.pop_symbol()?;
            match sym {
                ']' => nesting += 1,
                '[' => {
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
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn delimiterless_string() -> impl Strategy<Value = Vec<Symbol>> {
        any::<Vec<Symbol>>().prop_filter("symbol strings must not contain delimiters", |s| {
            !(s.contains(&'[') || s.contains(&']'))
        })
    }

    #[test]
    fn pop_string_fails_on_empty_stack() {
        let mut state = State::new(Interpreter::Null);

        assert!(state.pop_string().is_err());
    }

    #[test]
    fn pop_string_fails_on_delimiterless() {
        let mut state = State::new(Interpreter::Null);
        state.push_element(Element::Symbol('a'));

        assert!(state.pop_string().is_err());
    }

    proptest! {
        #[test]
        fn push_string_pop_string_succeeds(string in delimiterless_string()) {
            let mut state = State::new(Interpreter::Null);

            state.push_string(string.clone());
            state.pop_string()?;
        }

        #[test]
        fn push_string_pop_string_roundtrips(string in delimiterless_string()) {
            let mut state = State::new(Interpreter::Null);

            state.push_string(string.clone());
            let result = state.pop_string()?;

            prop_assert_eq!(result, string);
        }
    }
}
