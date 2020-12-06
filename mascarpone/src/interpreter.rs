use std::{
    collections::HashMap,
    io::{Read, Write},
    mem,
};

use crate::{
    operation::{Intrinsic, Operation},
    state::State,
    Error, Result, Symbol,
};

#[derive(Debug, Clone)]
enum Variant {
    Initial,
    QuoteString,
    QuoteSymbol,
    Mapping {
        mapping: HashMap<Symbol, Operation>,
        default: Operation,
    },
}

#[derive(Debug, Clone)]
pub enum Interpreter {
    Null,
    Defined {
        parent: Box<Interpreter>, // May need to use Rc instead.
        variant: Variant,
    },
}

impl Interpreter {
    fn from_variant(variant: Variant) -> Self {
        Self::Defined {
            parent: Box::new(Self::Null),
            variant,
        }
    }

    pub fn initial() -> Self {
        Self::from_variant(Variant::Initial)
    }

    pub fn quote_string() -> Self {
        Self::from_variant(Variant::QuoteString)
    }

    pub fn quote_symbol() -> Self {
        Self::from_variant(Variant::QuoteSymbol)
    }

    pub fn uniform(op: Operation) -> Self {
        Self::from_variant(Variant::Mapping {
            mapping: HashMap::new(),
            default: op,
        })
    }

    pub fn mapping(mapping: HashMap<Symbol, Operation>) -> Self {
        Self::from_variant(Variant::Mapping {
            mapping,
            default: Operation::Intrinsic(Intrinsic::NoOp),
        })
    }

    pub fn parent(&self) -> Option<&Self> {
        match self {
            Self::Null => None,
            Self::Defined { parent, .. } => Some(parent.as_ref()),
        }
    }

    pub fn set_parent(&mut self, new_parent: Interpreter) {
        if let Self::Defined { parent, .. } = self {
            mem::replace(parent.as_mut(), new_parent);
        }
    }

    pub fn extract(&self, sym: Symbol) -> Result<Operation> {
        match self {
            Self::Null => Err(Error::NullInterpreter),
            Self::Defined { variant, .. } => match variant {
                Variant::QuoteString | Variant::QuoteSymbol => Err(Error::WrongInterpreterVariant),
                Variant::Initial => Ok(Operation::Intrinsic(
                    Intrinsic::from_symbol(sym).unwrap_or(Intrinsic::NoOp),
                )),
                Variant::Mapping { mapping, default } => {
                    Ok(mapping.get(&sym).unwrap_or(default).clone())
                }
            },
        }
    }

    pub fn install(&mut self, sym: Symbol, op: Operation) -> Result<()> {
        match self {
            Self::Null => Err(Error::NullInterpreter),
            Self::Defined { variant, .. } => match variant {
                Variant::QuoteString | Variant::QuoteSymbol => Err(Error::WrongInterpreterVariant),
                Variant::Initial => {
                    let mut mapping = Operation::intrinsic_mapping();
                    mapping.insert(sym, op);
                    *variant = Variant::Mapping {
                        mapping,
                        default: Operation::Intrinsic(Intrinsic::NoOp),
                    };
                    Ok(())
                }
                Variant::Mapping { mapping, .. } => {
                    mapping.insert(sym, op);
                    Ok(())
                }
            },
        }
    }

    pub fn interpret<IO: Read + Write>(&self, sym: Symbol, state: &mut State<IO>) -> Result<()> {
        let op = self.extract(sym)?;
        op.execute(state)
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::initial()
    }
}
