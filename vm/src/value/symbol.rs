use std::fmt::{Debug, Formatter, Result as FmtResult};

#[derive(Clone, Eq, PartialEq, Hash, PartialOrd, Serialize, Deserialize)]
pub struct Symbol(pub String);

impl Debug for Symbol {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "'{}", self.0)
    }
}
