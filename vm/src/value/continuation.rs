use std::rc::Rc;
use super::FunctionPtr;
use super::Symbol;

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ContinuationPtr {
    pub function: FunctionPtr,
    pub parent: Option<Rc<ContinuationPtr>>,
    pub tag: Option<Symbol>,
}

impl ContinuationPtr {
    pub fn new(
        function: FunctionPtr,
        parent: Option<Rc<ContinuationPtr>>,
        symbol: Option<Symbol>,
    ) -> ContinuationPtr {
        ContinuationPtr {
            function,
            parent,
            tag: symbol,
        }
    }
}
