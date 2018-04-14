use std::rc::Rc;
use std::ops::Deref;

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ContinuationPtr {
    continuation: Rc<Continuation>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Continuation {}

impl ContinuationPtr {
    pub fn new(_c: Continuation) -> ContinuationPtr {
        unimplemented!()
    }
}

impl Deref for ContinuationPtr {
    type Target = Continuation;
    fn deref(&self) -> &Self::Target {
        unimplemented!()
    }
}
