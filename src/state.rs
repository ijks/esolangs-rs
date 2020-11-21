use crate::{
    interpreter::{Interpreter, Operation, PrimOp},
    io::SymbolIO,
    queue::Queue,
    stack::Stack,
    Program, Symbol,
};

#[derive(Debug, Clone)]
pub struct State<IO> {
    stack: Stack<Symbol>,
    queue: Queue<Symbol>,
    interpreter: Interpreter,
    pub io: IO,
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
        for &sym in program.next() {
            self.interpret_symbol(sym)?;
        }

        Ok(())
    }

    pub fn interpret_symbol(&mut self, sym: Symbol) -> Result<(), String> {
        let operation = self.interpreter.lookup(sym).clone();

        match operation {
            Operation::Primitive(primop) => self.step_primop(primop),
            Operation::Program(program) => self.run(&mut program.iter()),
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
