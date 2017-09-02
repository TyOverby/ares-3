use vm::Instruction;
use std::rc::Rc;

pub type FunctionPtr = Rc<Function>;

#[derive(Clone, PartialEq, Debug)]
pub struct Function {
    pub arg_count: usize,
    pub instructions: Vec<Instruction>,
}
