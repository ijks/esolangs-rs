use std::collections::HashMap;

use crate::{
    interpreter::Interpreter,
    state::{Element, State},
    Error, Result, Symbol,
};

#[derive(Debug, Clone)]
pub enum Operation {
    Intrinsic(Intrinsic),
    Program(Vec<Symbol>, Box<Interpreter>),
}

impl Operation {
    pub fn execute(&self, state: &mut State) -> Result<()> {
        match self {
            Self::Intrinsic(op) => op.execute(state),
            Self::Program(program, interp) => todo!(),
        }
    }

    pub fn intrinsic_mapping() -> HashMap<Symbol, Self> {
        Intrinsic::SYMBOLS
            .iter()
            .map(|&(op, sym)| (sym, Self::Intrinsic(op)))
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intrinsic {
    Reify,
    Deify,
    Extract,
    Install,
    GetParent,
    SetParent,
    Create,
    Expand,
    Perform,
    Null,
    Uniform,
    QuoteString,
    QuoteSymbol,
    Output,
    Input,
    Dup,
    Discard,
    Swap,
    NoOp,
}

impl Intrinsic {
    pub const SYMBOLS: [(Self, Symbol); 18] = [
        (Self::Reify, 'v'),
        (Self::Deify, '^'),
        (Self::Extract, '>'),
        (Self::Install, '<'),
        (Self::GetParent, '{'),
        (Self::SetParent, '}'),
        (Self::Create, '*'),
        (Self::Expand, '@'),
        (Self::Perform, '!'),
        (Self::Null, '0'),
        (Self::Uniform, '1'),
        (Self::QuoteString, '['),
        (Self::QuoteSymbol, '\''),
        (Self::Output, '.'),
        (Self::Input, ','),
        (Self::Dup, ':'),
        (Self::Discard, '$'),
        (Self::Swap, '/'),
    ];

    pub fn from_symbol(sym: Symbol) -> Option<Self> {
        Self::SYMBOLS
            .iter()
            .find(|&&(_, s)| s == sym)
            .map(|&(op, _)| op)
    }

    pub fn to_symbol(&self) -> Symbol {
        Self::SYMBOLS
            .iter()
            .find(|&&(op, _)| op == *self)
            .map(|&(_, sym)| sym)
            .expect("intrisic operation needs an associated symbol")
    }

    pub fn execute(&self, state: &mut State) -> Result<()> {
        match self {
            Self::Reify => {
                let interp = state.interpreter.clone();
                state.push_element(Element::Interpreter(interp));
                Ok(())
            }
            Self::Deify => {
                state.interpreter = state.pop_interpreter()?;
                Ok(())
            }
            Self::Extract => {
                let sym = state.pop_symbol()?;
                let interp = state.pop_interpreter()?;

                let op = interp.extract(sym)?;
                state.push_element(Element::Operation(op));
                Ok(())
            }
            Self::Install => {
                let sym = state.pop_symbol()?;
                let op = state.pop_operation()?;
                let mut interp = state.pop_interpreter()?;

                interp.install(sym, op)?;
                state.push_element(Element::Interpreter(interp));
                Ok(())
            }
            Self::GetParent => {
                let interpreter = state.pop_interpreter()?;
                let parent = interpreter.parent().ok_or(Error::NoParent)?.clone();
                state.push_element(Element::Interpreter(parent));
                Ok(())
            }
            Self::SetParent => {
                let (mut interp, new_parent) = (state.pop_interpreter()?, state.pop_interpreter()?);
                interp.set_parent(new_parent);
                state.push_element(Element::Interpreter(interp));
                Ok(())
            }
            Self::Create => {
                let interp = state.pop_interpreter()?;
                let program = state.pop_string()?;

                let op = Operation::Program(program, Box::new(interp));
                state.push_element(Element::Operation(op));
                Ok(())
            }
            Self::Expand => {
                let op = state.pop_operation()?;

                let (program, interp) = match op {
                    Operation::Intrinsic(op) => (vec![op.to_symbol()], Interpreter::initial()),
                    Operation::Program(program, interp) => (program, *interp),
                };

                state.push_string(program);
                state.push_element(Element::Interpreter(interp));
                Ok(())
            }
            Self::Perform => state.pop_operation()?.execute(state),
            Self::Null => {
                state.push_element(Element::Interpreter(Interpreter::Null));
                Ok(())
            }
            Self::Uniform => {
                let op = state.pop_operation()?;
                let interp = Interpreter::uniform(op);
                state.push_element(Element::Interpreter(interp));
                Ok(())
            }
            Self::QuoteString => {
                state.push_element(Element::Symbol(crate::STRING_LEFT_DELIM));
                state.push_element(Element::Interpreter(Interpreter::quote_string()));
                Ok(())
            }
            Self::QuoteSymbol => {
                state.push_element(Element::Interpreter(Interpreter::quote_symbol()));
                Ok(())
            }
            Self::Input => todo!(),
            Self::Output => todo!(),
            Self::Dup => {
                let elem = state.peek_element()?.clone();
                state.push_element(elem);
                Ok(())
            }
            Self::Discard => {
                // The spec doesn't specify if discarding from an empty stack is an error,
                // so I've decided to be strict in that case, for the time being.
                let _ = state.pop_element()?;
                Ok(())
            }
            Self::Swap => {
                let (a, b) = (state.pop_element()?, state.pop_element()?);
                state.push_element(a);
                state.push_element(b);
                Ok(())
            }
            Self::NoOp => Ok(()),
        }
    }
}
