use vm::{Symbol, VmError,  VmResult};
use function::FunctionPtr;
use continuation::ContinuationPtr;

#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Symbol(Symbol),
    Function(FunctionPtr),
    Continuation(ContinuationPtr)
}

impl Value {
    //
    // TO
    //
    pub fn to_int(self) -> VmResult<i64> {
        match self {
            Value::Integer(i) => Ok(i),
            other => Err(VmError::UnexpectedType(other)),
        }
    }

    pub fn to_float(self) -> VmResult<f64> {
        match self {
            Value::Float(f) => Ok(f),
            other => Err(VmError::UnexpectedType(other)),
        }
    }

    pub fn to_symbol(self) -> VmResult<Symbol> {
        match self {
            Value::Symbol(s) => Ok(s),
            other => Err(VmError::UnexpectedType(other)),
        }
    }

    pub fn to_function(self) -> VmResult<FunctionPtr> {
        match self {
            Value::Function(f) => Ok(f),
            other => Err(VmError::UnexpectedType(other)),
        }
    }

    pub fn to_continuation(self) -> VmResult<ContinuationPtr> {
        match self {
            Value::Continuation(c) => Ok(c),
            other => Err(VmError::UnexpectedType(other)),
        }
    }

    //
    // AS
    //
    pub fn as_int(&self) -> VmResult<&i64> {
        match self {
            &Value::Integer(ref i) => Ok(i),
            other => Err(VmError::UnexpectedType(other.clone())),
        }
    }

    pub fn as_float(&self) -> VmResult<&f64> {
        match self {
            &Value::Float(ref f) => Ok(f),
            other => Err(VmError::UnexpectedType(other.clone())),
        }
    }

    pub fn as_symbol(&self) -> VmResult<&Symbol> {
        match self {
            &Value::Symbol(ref s) => Ok(s),
            other => Err(VmError::UnexpectedType(other.clone())),
        }
    }

    pub fn as_function(&self) -> VmResult<&FunctionPtr> {
        match self {
            &Value::Function(ref f) => Ok(f),
            other => Err(VmError::UnexpectedType(other.clone())),
        }
    }

    pub fn as_continuation(&self) -> VmResult<&ContinuationPtr> {
        match self {
            &Value::Continuation(ref c) => Ok(c),
            other => Err(VmError::UnexpectedType(other.clone())),
        }
    }
}
