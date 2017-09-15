use vm::{Symbol, VmError, VmResult};
use function::FunctionPtr;
use continuation::ContinuationPtr;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use hamt_rs::{HamtMap, CopyStore};
use std::hash::{Hash, Hasher};

pub type AresMap = HamtMap<Value, Value, CopyStore<Value, Value>>;

#[derive(Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Symbol(Symbol),
    Function(FunctionPtr),
    Continuation(ContinuationPtr),
    Map(AresMap),
}

impl Hash for Value {
    fn hash<H>(&self, state: &mut H)
    where H: Hasher {
        match self {
            &Value::Integer(i) => i.hash(state),
            &Value::Float(f) => {
                let as_i: u64 = unsafe { ::std::mem::transmute(f) };
                as_i.hash(state);
            }
            &Value::Symbol(ref s) => s.hash(state),

            &Value::Function(_) |
            &Value::Continuation(_) |
            &Value::Map(_) => {
                unimplemented!();
            }
        }
    }
}
impl Eq for Value {}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ValueKind {
    Integer,
    Float,
    Symbol,
    Function,
    Continuation,
    Map,
}

impl Debug for Value {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            &Value::Integer(i) => write!(f, "{}i64", i),
            &Value::Float(n) => write!(f, "{}f64", n),
            &Value::Symbol(Symbol(s)) => write!(f, "'{}", s),
            &Value::Function(ref c) => write!(f, "{:?}", c.borrow()),
            &Value::Continuation(ref c) => {
                if f.alternate() {
                    write!(f, "{:?}", c)
                } else {
                    write!(f, "<continuation>")
                }
            }
            &Value::Map(ref m) => {
                write!(f, "{{")?;
                for (k, v) in m.iter() {
                    write!(f, "{:?}: {:?},", k, v)?;
                }
                write!(f, "}}")
            }
        }
    }
}

impl Value {
    //
    // TO
    //
    pub fn to_int(self) -> VmResult<i64> {
        match self {
            Value::Integer(i) => Ok(i),
            other => Err(VmError::UnexpectedType {
                found: other,
                expected: ValueKind::Integer,
            }),
        }
    }

    pub fn to_float(self) -> VmResult<f64> {
        match self {
            Value::Float(f) => Ok(f),
            other => Err(VmError::UnexpectedType {
                found: other,
                expected: ValueKind::Float,
            }),
        }
    }

    pub fn to_symbol(self) -> VmResult<Symbol> {
        match self {
            Value::Symbol(s) => Ok(s),
            other => Err(VmError::UnexpectedType {
                found: other,
                expected: ValueKind::Symbol,
            }),
        }
    }

    pub fn to_function(self) -> VmResult<FunctionPtr> {
        match self {
            Value::Function(f) => Ok(f),
            other => Err(VmError::UnexpectedType {
                found: other,
                expected: ValueKind::Function,
            }),
        }
    }

    pub fn to_continuation(self) -> VmResult<ContinuationPtr> {
        match self {
            Value::Continuation(c) => Ok(c),
            other => Err(VmError::UnexpectedType {
                found: other,
                expected: ValueKind::Continuation,
            }),
        }
    }

    pub fn to_map(self) -> VmResult<AresMap> {
        match self {
            Value::Map(c) => Ok(c),
            other => Err(VmError::UnexpectedType {
                found: other,
                expected: ValueKind::Map,
            }),
        }
    }

    //
    // AS
    //
    pub fn as_int(&self) -> VmResult<&i64> {
        match self {
            &Value::Integer(ref i) => Ok(i),
            other => Err(VmError::UnexpectedType {
                found: other.clone(),
                expected: ValueKind::Integer,
            }),
        }
    }

    pub fn as_float(&self) -> VmResult<&f64> {
        match self {
            &Value::Float(ref f) => Ok(f),
            other => Err(VmError::UnexpectedType {
                found: other.clone(),
                expected: ValueKind::Float,
            }),
        }
    }

    pub fn as_symbol(&self) -> VmResult<&Symbol> {
        match self {
            &Value::Symbol(ref s) => Ok(s),
            other => Err(VmError::UnexpectedType {
                found: other.clone(),
                expected: ValueKind::Symbol,
            }),
        }
    }

    pub fn as_function(&self) -> VmResult<&FunctionPtr> {
        match self {
            &Value::Function(ref f) => Ok(f),
            other => Err(VmError::UnexpectedType {
                found: other.clone(),
                expected: ValueKind::Function,
            }),
        }
    }

    pub fn as_continuation(&self) -> VmResult<&ContinuationPtr> {
        match self {
            &Value::Continuation(ref c) => Ok(c),
            other => Err(VmError::UnexpectedType {
                found: other.clone(),
                expected: ValueKind::Continuation,
            }),
        }
    }

    pub fn as_map(&self) -> VmResult<&AresMap> {
        match self {
            &Value::Map(ref c) => Ok(c),
            other => Err(VmError::UnexpectedType {
                found: other.clone(),
                expected: ValueKind::Map,
            }),
        }
    }
}
