#![feature(never_type)]

use std::{
    collections::{HashMap, VecDeque},
    error::Error,
    io::{self, Read, Write},
};

type Symbol = u8;

#[derive(Debug, Clone, Copy)]
enum PrimOp {
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
enum Operation {
    Primitive(PrimOp),
    Program(Vec<Symbol>),
    NoOp,
}

#[derive(Debug, Clone)]
struct Program<'a> {
    contents: &'a [Symbol],
    position: usize,
}

impl<'a> Program<'a> {
    pub fn new(contents: &'a [Symbol]) -> Self {
        Self {
            contents,
            position: 0,
        }
    }

    pub fn advance(&mut self) -> Option<&Symbol> {
        let old_position = self.position;
        self.position += 1;
        self.contents.get(old_position)
    }
}

#[derive(Debug, Clone)]
struct Interpreter {
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

#[derive(Debug, Clone)]
struct Stack<T> {
    storage: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Stack {
            storage: Vec::new(),
        }
    }

    pub fn pop(&mut self) -> Result<T, String> {
        self.storage.pop().ok_or("stack is empty".into())
    }

    pub fn pop_string(&mut self, terminator: T) -> Result<Vec<T>, String>
    where
        T: PartialEq<T>,
    {
        let mut string = VecDeque::new();
        while let Some(item) = self.storage.pop() {
            if item == terminator {
                return Ok(string.into());
            }

            string.push_front(item)
        }

        Err("prematurely terminated string".into())
    }

    pub fn push(&mut self, value: T) {
        self.storage.push(value)
    }

    pub fn peek(&self) -> Result<&T, String> {
        self.storage.last().ok_or("stack is empty".into())
    }
}

type Queue<T> = VecDeque<T>;

#[derive(Debug, Clone)]
struct State<IO> {
    stack: Stack<Symbol>,
    queue: Queue<Symbol>,
    interpreter: Interpreter,
    io: IO,
}

trait SymbolIO {
    type Error: Error;

    fn read_symbol(&mut self) -> Result<Symbol, Self::Error>;
    fn write_symbol(&mut self, sym: Symbol) -> Result<(), Self::Error>;
}

impl<IO: SymbolIO> State<IO> {
    pub fn new(interpreter: Interpreter, io: IO) -> Self {
        Self {
            stack: Stack::new(),
            queue: Queue::new(),
            interpreter,
            io,
        }
    }

    pub fn run(&mut self, program: &mut Program) -> Result<(), String> {
        while let Some(&sym) = program.advance() {
            self.interpret_symbol(sym)?;
        }

        Ok(())
    }

    pub fn interpret_symbol(&mut self, sym: Symbol) -> Result<(), String> {
        let operation = self.interpreter.lookup(sym).clone();

        match operation {
            Operation::Primitive(primop) => self.step_primop(primop),
            Operation::Program(program) => {
                let mut program = Program::new(program.as_slice());
                self.run(&mut program)
            }
            Operation::NoOp => Ok(()),
        }
    }

    pub fn step_primop(&mut self, primop: PrimOp) -> Result<(), String> {
        Ok(match primop {
            PrimOp::Nul => self.stack.push(0),
            PrimOp::Semicolon => self.stack.push(b';'),
            PrimOp::Digit(d) => {
                let sym = self.stack.pop()?;
                self.stack.push(sym.wrapping_mul(10).wrapping_add(d));
            }
            PrimOp::Add => {
                let rhs = self.stack.pop()?;
                let lhs = self.stack.pop()?;
                self.stack.push(lhs.wrapping_add(rhs))
            }
            PrimOp::Sub => {
                let rhs = self.stack.pop()?;
                let lhs = self.stack.pop()?;
                self.stack.push(lhs.wrapping_sub(rhs))
            }
            PrimOp::Log2 => {
                let sym = self.stack.pop()?;
                self.stack.push(match sym {
                    0 => 8,
                    n => (n as f64).log2().floor() as u8,
                })
            }
            PrimOp::Output => {
                let sym = self.stack.pop()?;
                self.io.write_symbol(sym).map_err(|e| e.to_string())?
            }
            PrimOp::Input => {
                let sym = self.io.read_symbol().map_err(|e| e.to_string())?;
                self.stack.push(sym)
            }
            PrimOp::Enqueue => {
                let sym = self.stack.peek()?;
                self.queue.push_back(*sym)
            }
            PrimOp::Dequeue => {
                let sym = self.queue.pop_front().ok_or("queue is empty")?;
                self.stack.push(sym)
            }
            PrimOp::Duplicate => {
                let sym = self.stack.peek()?;
                self.stack.push(*sym)
            }
            PrimOp::Supplant => {
                let sym = self.stack.pop()?;
                let program = self.stack.pop_string(b';')?;
                self.interpreter.supplant(sym, Operation::Program(program))
            }
            PrimOp::Eval => {
                let sym = self.stack.pop()?;
                self.interpret_symbol(sym)?
            }
        })
    }
}

struct StandardIO;

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

struct StringIO<'s> {
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

fn run_with_io<IO: SymbolIO>(io: IO, program: &[Symbol]) -> Result<State<IO>, String> {
    let mut program = Program::new(program);
    let mut state = State::new(Interpreter::default(), io);
    state.run(&mut program)?;
    Ok(state)
}

fn run_with_input(program: &[Symbol], input: &[Symbol]) -> Result<Vec<Symbol>, String> {
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
