use vm::Instruction;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter, Result as FmtResult};

pub type FunctionPtr = Rc<RefCell<Function>>;

pub fn new_func(f: Function) -> FunctionPtr {
    Rc::new(RefCell::new(f))
}

#[derive(Clone, PartialEq)]
pub struct Function {
    pub name: Option<String>,
    pub arg_count: usize,
    pub instructions: Vec<Instruction>,
}

impl Debug for Function {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let name = if let &Some(ref n) = &self.name {
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
