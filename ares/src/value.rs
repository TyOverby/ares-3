use vm::{Symbol, VmError,  VmResult};
use function::FunctionPtr;

#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Symbol(Symbol),
    Function(FunctionPtr),
}

impl Value {
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
}
