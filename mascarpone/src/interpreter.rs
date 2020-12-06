use std::{
    collections::HashMap,
    io::{Read, Write},
};

use crate::{
    operation::{Intrinsic, Operation},
    state::Element,
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

// Note that there's no explicit "null interpreter"; instead, the parent interpreter is
// optional, as are interpreter values on the stack. This means we diverge slightly from
// what the spec explicitly describes, in that e.g. trying to deify a null interpreter is
// already an error, rather than trying to execute an operation with it. Practically,
// there's not a real difference.
#[derive(Debug, Clone)]
pub struct Interpreter {
    parent: Option<Box<Interpreter>>, // May need to use Rc instead.
    variant: Variant,
}

impl Interpreter {
    fn new(variant: Variant) -> Self {
        Self {
            parent: None,
            variant,
        }
    }

    pub fn initial() -> Self {
        Self::new(Variant::Initial)
    }

    pub fn quote_string() -> Self {
        Self::new(Variant::QuoteString)
    }

    pub fn quote_symbol() -> Self {
        Self::new(Variant::QuoteSymbol)
    }

    pub fn uniform(op: Operation) -> Self {
        Self::new(Variant::Mapping {
            mapping: HashMap::new(),
            default: op,
        })
    }

    pub fn mapping(mapping: HashMap<Symbol, Operation>) -> Self {
        Self::new(Variant::Mapping {
            mapping,
            default: Operation::Intrinsic(Intrinsic::NoOp),
        })
    }

    // TODO: factor out a `DefinedInterpreter` or the like, so we can just
    // access the parent as a field.
    pub fn parent(&self) -> Option<&Self> {
        self.parent.as_deref()
    }

    pub fn set_parent(&mut self, parent: Option<Interpreter>) {
        self.parent = parent.map(Box::new);
    }

    pub fn variant(&self) -> &Variant {
        &self.variant
    }

    pub fn extract(&self, sym: Symbol) -> Result<Operation> {
        match self.variant {
            Variant::QuoteString | Variant::QuoteSymbol => Err(Error::WrongInterpreterVariant),
            Variant::Initial => Ok(Operation::Intrinsic(
                Intrinsic::from_symbol(sym).unwrap_or(Intrinsic::NoOp),
            )),
            Variant::Mapping {
                ref mapping,
                ref default,
            } => Ok(mapping.get(&sym).unwrap_or(default).clone()),
        }
    }

    pub fn install(&mut self, sym: Symbol, op: Operation) -> Result<()> {
        match self.variant {
            Variant::QuoteString | Variant::QuoteSymbol => Err(Error::WrongInterpreterVariant),
            Variant::Initial => {
                let mut mapping = Operation::intrinsic_mapping();
                mapping.insert(sym, op);
                self.variant = Variant::Mapping {
                    mapping,
                    default: Operation::Intrinsic(Intrinsic::NoOp),
                };
                Ok(())
            }
            Variant::Mapping {
                ref mut mapping, ..
            } => {
                mapping.insert(sym, op);
                Ok(())
            }
        }
    }

    pub fn interpret<IO: Read + Write>(&self, sym: Symbol, state: &mut State<IO>) -> Result<()> {
        match self.variant {
            Variant::QuoteString => {
                state.push_element(Element::Symbol(sym));

                match sym {
                    crate::STRING_RIGHT_DELIM => {
                        // TODO: put this in `state.switch_to_parent` or something
                        state.interpreter = self.parent().expect("parent should be defined").clone()
                    }
                    crate::STRING_LEFT_DELIM => state.start_quote_string(),
                    _ => (),
                }

                Ok(())
            }
            Variant::QuoteSymbol => {
                state.push_element(Element::Symbol(sym));
                state.interpreter = self.parent().expect("parent should be defined").clone();
                Ok(())
            }
            Variant::Initial => Intrinsic::from_symbol(sym)
                .unwrap_or(Intrinsic::NoOp)
                .execute(state),
            Variant::Mapping {
                ref mapping,
                ref default,
            } => mapping.get(&sym).unwrap_or(&default).execute(state),
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::initial()
    }
}
