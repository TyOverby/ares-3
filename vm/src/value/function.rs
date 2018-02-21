use vm::Instruction;
use std::rc::Rc;
use std::ops::Deref;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter, Result as FmtResult};

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct FunctionPtr {
    function: Rc<RefCell<Function>>,
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

#[derive(PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Function {
    pub name: Option<String>,
    pub arg_count: usize,
    pub instructions: Vec<Instruction>,
}

impl Debug for Function {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let name = if let Some(ref n) = self.name {
            &n[..]
        } else {
            "<unnamed>"
        };

        f.debug_struct("Function")
            .field("name", &name)
            .field("arg_count", &self.arg_count)
            .finish()
    }
}
