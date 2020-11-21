use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct Stack<T> {
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
