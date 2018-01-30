use vm::ValueStack;
use std::rc::Rc;

pub type ContinuationPtr = Rc<Continuation>;

#[derive(Clone, Debug, PartialEq)]
pub struct Continuation {
    pub(crate) stack: ValueStack,
}
