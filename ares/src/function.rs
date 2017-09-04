use vm::Instruction;
use std::rc::Rc;
use std::cell::RefCell;
use value::Value;
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
        writeln!(f, "Function {{")?;
        if let &Some(ref name) = &self.name {
            writeln!(f, "  {}", name)?;
        } else {
            writeln!(f, "  <no name>")?;
        }
        writeln!(f, "  {} arguments", self.arg_count)?;
        writeln!(f, "  instructions: [")?;
        for instr in &self.instructions {
            match instr {
                &Instruction::Push(Value::Function(ref funk)) => {
                    if let &Some(ref name) = &funk.borrow().name {
                        writeln!(f, "    Push(Value(function {:?}))", name)?;
                    } else {
                        writeln!(f, "    Push(Value(function <no name>))")?;
                    }
                }
                other =>
                    writeln!(f, "    {:?}", other)?,
            }
        }
        writeln!(f, "  ]")?;
        writeln!(f, "}}")?;
        Ok(())
    }
}
