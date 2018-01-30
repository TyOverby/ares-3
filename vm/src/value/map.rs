use rpds::RedBlackTreeMap;
use super::Value;
use std::ops::Deref;

#[derive(Clone, PartialEq, PartialOrd)]
pub struct AresMap {
    map: RedBlackTreeMap<Value, Value>,
}

impl AresMap {
    pub fn new() -> AresMap {
        AresMap {
            map: RedBlackTreeMap::new(),
        }
    }

    pub fn insert(&self, key: Value, value: Value) -> AresMap {
        AresMap {
            map: self.map.insert(key, value),
        }
    }
}

impl Deref for AresMap {
    type Target = RedBlackTreeMap<Value, Value>;
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}
