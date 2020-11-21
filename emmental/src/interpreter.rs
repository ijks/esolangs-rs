use std::collections::HashMap;

use crate::Symbol;

#[derive(Debug, Clone, Copy)]
pub enum PrimOp {
    Nul,       // #
    Digit(u8), // TODO: more restricted type, just for 0...9
    Add,       // +
    Sub,       // -
    Log2,      // ~
    Output,    // .
    Input,     // ,
    Enqueue,   // ^
    Dequeue,   // v
    Duplicate, // :
    Supplant,  // !
    Eval,      // ?
    Semicolon, // ;
}

#[derive(Debug, Clone)]
pub enum Operation {
    Primitive(PrimOp),
    Program(Vec<Symbol>),
    NoOp,
}

#[derive(Debug, Clone)]
pub struct Interpreter {
    map: HashMap<Symbol, Operation>,
}

impl Interpreter {
    pub fn lookup(&self, sym: Symbol) -> &Operation {
        self.map.get(&sym).unwrap_or(&Operation::NoOp)
    }

    pub fn supplant(&mut self, sym: Symbol, op: Operation) {
        self.map.insert(sym, op);
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        const DIGIT_OFFSET: u8 = 48;
        let digits = (0..=9).map(|d| (DIGIT_OFFSET + d, PrimOp::Digit(d)));
        let rest = [
            (b'#', PrimOp::Nul),
            (b'+', PrimOp::Add),
            (b'-', PrimOp::Sub),
            (b'~', PrimOp::Log2),
            (b'.', PrimOp::Output),
            (b',', PrimOp::Input),
            (b'^', PrimOp::Enqueue),
            (b'v', PrimOp::Dequeue),
            (b':', PrimOp::Duplicate),
            (b'!', PrimOp::Supplant),
            (b'?', PrimOp::Eval),
            (b';', PrimOp::Semicolon),
        ];
        let map = digits
            .chain(rest.iter().cloned())
            .map(|(k, v)| (k, Operation::Primitive(v)))
            .collect();
        Self { map }
    }
}
