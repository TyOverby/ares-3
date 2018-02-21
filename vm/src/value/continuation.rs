use vm::ValueStack;
use std::rc::Rc;
use std::ops::Deref;

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ContinuationPtr {
    continuation: Rc<Continuation>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Continuation {
    pub(crate) stack: ValueStack,
}

impl ContinuationPtr {
    pub fn new(c: Continuation) -> ContinuationPtr {
        ContinuationPtr {
            continuation: Rc::new(c),
        }
    }
}

impl Deref for ContinuationPtr{
    type Target = Continuation;
    fn deref(&self) -> &Self::Target {
        &self.continuation
    }
}
