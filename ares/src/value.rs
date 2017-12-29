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
            Value::Function(ref c) => if f.alternate() {
                write!(f, "{:?}", c.borrow())
            } else {
                let func = c.borrow();
                let name = func.name.as_ref().map(|s| s.as_ref());
                write!(f, "function {}", name.unwrap_or("<unnamed>"))
            },
            Value::Continuation(ref c) => if f.alternate() {
                write!(f, "{:?}", c)
            } else {
                write!(f, "<continuation>")
            },
            Value::Obj(ref o) => {
                write!(f, "Object {{")?;
                if f.alternate() {
                    write!(f, "\n")?
                }
                for (&Symbol(k), v) in o.iter() {
                    write!(f, "{:?}: {:?},", k, v)?;
                    if f.alternate() {
                        write!(f, "\n")?
                    }
                }
                if f.alternate() {
                    write!(f, "\n")?
                }
                write!(f, "}}")
            }
            Value::Map(ref o) => {
                write!(f, "Map {{")?;
                if f.alternate() {
                    write!(f, "\n")?
                }
                for (k, v) in o.iter() {
                    write!(f, "{:?}: {:?},", k, v)?;
                    if f.alternate() {
                        write!(f, "\n")?
                    }
                }
                if f.alternate() {
                    write!(f, "\n")?
                }
                write!(f, "}}")
            }
        }
    }
}

macro_rules! impl_for_variant {
    ($is_name: ident, $to_name: ident, $as_name: ident, $variant: ident, $typ: ty) => {
        pub fn $is_name(self) -> bool {
            match self {
                Value::$variant(_) =>  true,
                _ => false
            }
        }

        pub fn $to_name(self) -> VmResult<$typ> {
            match self {
                Value::$variant(i) => Ok(i),
                other => Err(VmError::UnexpectedType {
                    expected: ValueKind::$variant,
                    found: other,
                }),
            }
        }

        pub fn $as_name(&self) -> VmResult<&$typ> {
            match self {
                &Value::$variant(ref i) => Ok(i),
                other => Err(VmError::UnexpectedType {
                    expected: ValueKind::$variant,
                    found: other.clone(),
                }),
            }
        }
    };
}

impl Value {
    impl_for_variant!(is_int, into_int, as_int, Integer, i64);
    impl_for_variant!(is_float, into_float, as_float, Float, f64);
    impl_for_variant!(is_symbol, into_symbol, as_symbol, Symbol, Symbol);
    impl_for_variant!(is_function, into_function, as_function, Function, FunctionPtr);
    impl_for_variant!(is_continuation, into_continuation, as_continuation, Continuation, ContinuationPtr);
    impl_for_variant!(is_map, into_map, as_map, Map, AresMap);
    impl_for_variant!(is_obj, into_obj, as_obj, Obj, AresObj);
}
