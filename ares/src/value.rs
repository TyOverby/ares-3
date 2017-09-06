use vm::{Symbol, VmError, VmResult};
use function::FunctionPtr;
use continuation::ContinuationPtr;
use std::fmt::{Debug, Formatter, Result as FmtResult};

#[derive(Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Symbol(Symbol),
    Function(FunctionPtr),
    Continuation(ContinuationPtr),
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ValueKind {
    Integer,
    Float,
    Symbol,
    Function,
    Continuation,
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
}
