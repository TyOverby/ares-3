use rpds::List;
use super::Value;
use std::ops::Deref;

#[derive(Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AresList {
    list: List<Value>,
}

impl Deref for AresList {
    type Target = List<Value>;
    fn deref(&self) -> &Self::Target {
        &self.list
    }
}
