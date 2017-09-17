use vm::{Symbol, VmError, VmResult};
use function::FunctionPtr;
use continuation::ContinuationPtr;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use hamt_rs::{CopyStore, HamtMap};
use std::hash::{Hash, Hasher};

pub type AresMap = HamtMap<Value, Value, CopyStore<Value, Value>>;
pub type AresObj = HamtMap<Symbol, Value, CopyStore<Symbol, Value>>;

#[derive(Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Symbol(Symbol),
    Function(FunctionPtr),
    Continuation(ContinuationPtr),
    Map(AresMap),
    Obj(AresObj),
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        use self::Value::*;
        match (self, other) {
            (&Integer(l), &Integer(r)) => l == r,
            (&Float(l), &Float(r)) => l == r,
            (&Symbol(ref l), &Symbol(ref r)) => l == r,
            (&Function(ref l), &Function(ref r)) => l == r,
            (&Continuation(ref l), &Continuation(ref r)) => l == r,
            (&Map(ref l), &Map(ref r)) => l == r,
            (&Obj(ref l), &Obj(ref r)) => l == r,
            _ => false,
        }
    }
}

impl Hash for Value {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        match *self {
            Value::Integer(i) => i.hash(state),
            Value::Float(f) => {
                let as_i: u64 = unsafe { ::std::mem::transmute(f) };
                as_i.hash(state);
            }
            Value::Symbol(ref s) => s.hash(state),
            Value::Function(_) | Value::Continuation(_) | Value::Obj(_) | Value::Map(_) => {
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
    Obj,
}

impl Debug for Value {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            Value::Integer(i) => write!(f, "{}i64", i),
            Value::Float(n) => write!(f, "{}f64", n),
            Value::Symbol(Symbol(s)) => write!(f, "'{}", s),
            Value::Function(ref c) => write!(f, "{:?}", c.borrow()),
            Value::Continuation(ref c) => if f.alternate() {
                write!(f, "{:?}", c)
            } else {
                write!(f, "<continuation>")
            },
            Value::Obj(ref o) => {
                write!(f, "Object {{")?;
                for (&Symbol(k), v) in o.iter() {
                    write!(f, "{:?}: {:?},", k, v)?;
                }
                write!(f, "}}")
            }
            Value::Map(ref o) => {
                write!(f, "Map {{")?;
                for (k, v) in o.iter() {
                    write!(f, "{:?}: {:?},", k, v)?;
                }
                write!(f, "}}")
            }
        }
    }
}

impl Value {
    //
    // Is
    //
    pub fn is_int(&self) -> bool {
        match *self {
            Value::Integer(_) => true,
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match *self {
            Value::Float(_) => true,
            _ => false,
        }
    }

    pub fn is_symbol(&self) -> bool {
        match *self {
            Value::Symbol(_) => true,
            _ => false,
        }
    }

    pub fn is_function(&self) -> bool {
        match *self {
            Value::Function(_) => true,
            _ => false,
        }
    }

    pub fn is_continuation(&self) -> bool {
        match *self {
            Value::Continuation(_) => true,
            _ => false,
        }
    }

    pub fn is_map(&self) -> bool {
        match *self {
            Value::Map(_) => true,
            _ => false,
        }
    }

    pub fn is_obj(&self) -> bool {
        match *self {
            Value::Obj(_) => true,
            _ => false,
        }
    }
    //
    // TO
    //
    pub fn into_int(self) -> VmResult<i64> {
        match self {
            Value::Integer(i) => Ok(i),
            other => Err(VmError::UnexpectedType {
                found: other,
                expected: ValueKind::Integer,
            }),
        }
    }

    pub fn into_float(self) -> VmResult<f64> {
        match self {
            Value::Float(f) => Ok(f),
            other => Err(VmError::UnexpectedType {
                found: other,
                expected: ValueKind::Float,
            }),
        }
    }

    pub fn into_symbol(self) -> VmResult<Symbol> {
        match self {
            Value::Symbol(s) => Ok(s),
            other => Err(VmError::UnexpectedType {
                found: other,
                expected: ValueKind::Symbol,
            }),
        }
    }

    pub fn into_function(self) -> VmResult<FunctionPtr> {
        match self {
            Value::Function(f) => Ok(f),
            other => Err(VmError::UnexpectedType {
                found: other,
                expected: ValueKind::Function,
            }),
        }
    }

    pub fn into_continuation(self) -> VmResult<ContinuationPtr> {
        match self {
            Value::Continuation(c) => Ok(c),
            other => Err(VmError::UnexpectedType {
                found: other,
                expected: ValueKind::Continuation,
            }),
        }
    }

    pub fn into_map(self) -> VmResult<AresMap> {
        match self {
            Value::Map(c) => Ok(c),
            other => Err(VmError::UnexpectedType {
                found: other,
                expected: ValueKind::Map,
            }),
        }
    }

    pub fn into_obj(self) -> VmResult<AresObj> {
        match self {
            Value::Obj(c) => Ok(c),
            other => Err(VmError::UnexpectedType {
                found: other,
                expected: ValueKind::Obj,
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

    pub fn as_obj(&self) -> VmResult<&AresObj> {
        match self {
            &Value::Obj(ref c) => Ok(c),
            other => Err(VmError::UnexpectedType {
                found: other.clone(),
                expected: ValueKind::Obj,
            }),
        }
    }
}
