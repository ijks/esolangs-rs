use thiserror::Error;

mod interpreter;
mod operation;
mod stack;
mod state;

pub type Symbol = char;

const STRING_LEFT_DELIM: Symbol = '[';
const STRING_RIGHT_DELIM: Symbol = ']';

#[derive(Error, Debug, Clone)]
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
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
