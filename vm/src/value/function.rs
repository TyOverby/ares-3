use vm::Instruction;
use super::Value;
use super::Symbol;
use std::rc::Rc;
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct FunctionPtr {
    pub function: Rc<Function>,
}

impl Deref for FunctionPtr {
    type Target = Function;
    fn deref(&self) -> &Self::Target {
        &self.function
    }
}

pub fn new_func(f: Function) -> FunctionPtr {
    FunctionPtr {
        function: Rc::new(f),
    }
}

#[derive(PartialEq, Debug, PartialOrd, Serialize, Deserialize, Clone)]
pub struct Function {
    pub name: Option<String>,
    pub instructions: Vec<Instruction>,
    pub is_built: bool,
    pub built: BuiltFunction,

    pub args_count: u32,
    pub upvars_count: u32,
    pub locals_count: u32,
}

#[derive(PartialEq, Debug, PartialOrd, Serialize, Deserialize, Clone)]
pub struct BuiltFunction {
    pub upvars: Vec<Value>,
    pub continuation: Option<(FunctionPtr, Option<Symbol>)>,
}

impl Function {
    pub fn tag(&self) -> Option<Symbol> {
        self.built
            .continuation.as_ref()
            .and_then(|&(_, ref tag)| tag.clone())
    }
}

impl FunctionPtr {
    pub fn continuation(&self) -> Option<(FunctionPtr, Option<Symbol>)> {
        self.function.built.continuation.clone()
    }

    pub fn tag(&self) -> Option<Symbol> {
        self.continuation().and_then(|(_, tag)| tag)
    }

    pub fn with_opt_continuation(
        mut self,
        c: Option<(FunctionPtr, Option<Symbol>)>,
    ) -> FunctionPtr {
        assert!(self.is_built);
        Rc::make_mut(&mut self.function)
            .built
            .continuation = c;
        self
    }

    pub fn with_continuation(self, c: (FunctionPtr, Option<Symbol>)) -> FunctionPtr {
        self.with_opt_continuation(Some(c))
    }
}
