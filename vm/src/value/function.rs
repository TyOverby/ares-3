use vm::Instruction;
use super::Value;
use std::rc::Rc;
use std::ops::Deref;
use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct FunctionPtr {
    pub function: Rc<RefCell<Function>>,
}

impl Deref for FunctionPtr {
    type Target = RefCell<Function>;
    fn deref(&self) -> &Self::Target {
        &self.function
    }
}

pub fn new_func(f: Function) -> FunctionPtr {
    FunctionPtr {
        function: Rc::new(RefCell::new(f)),
    }
}

#[derive(PartialEq, Debug, PartialOrd, Serialize, Deserialize, Clone)]
pub struct Function {
    pub name: Option<String>,
    pub instructions: Vec<Instruction>,
    pub upvars: Vec<Value>,

    pub args_count: u32,
    pub upvars_count: u32,
    pub locals_count: u32,
}
