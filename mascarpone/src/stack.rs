use std::iter;

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

    pub fn pop(&mut self) -> Option<T> {
        self.storage.pop()
    }

    pub fn pop_while<'a, P>(&'a mut self, mut pred: P) -> impl Iterator<Item = T> + 'a
    where
        P: FnMut(&T) -> bool + 'a,
    {
        iter::from_fn(move || {
            let x = self.pop()?;
            if pred(&x) {
                Some(x)
            } else {
                None
            }
        })
    }

    pub fn push(&mut self, value: T) {
        self.storage.push(value)
    }

    pub fn peek(&self) -> Option<&T> {
        self.storage.last()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pop_while_empty() {
        let mut stack = Stack::<u32>::new();

        let mut iter = stack.pop_while(|_| true);

        assert_eq!(iter.next(), None)
    }

    #[test]
    fn pop_while_const_false() {
        let mut stack = Stack {
            storage: vec![1u32, 2, 3],
        };

        let mut iter = stack.pop_while(|_| false);

        assert_eq!(iter.next(), None)
    }

    #[test]
    fn pop_while() {
        let mut stack = Stack {
            storage: vec![1u32, 2, 3],
        };

        let iter = stack.pop_while(|&n| n > 1u32);

        assert_eq!(iter.collect::<Vec<_>>(), vec![3, 2])
    }
}
