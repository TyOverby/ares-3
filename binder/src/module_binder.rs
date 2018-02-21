use super::*;
use std::collections::HashSet;

#[derive(Debug)]
pub struct ModuleBinder {
    pub module_id: u32,
    pub definitions: HashSet<String>,
}

impl<'bound> Binder<'bound> for ModuleBinder {
    fn add_declaration(&mut self, symbol: &'bound str) -> BindingKind {
        self.definitions.insert(symbol.into());
        BindingKind::Module {
            module_id: self.module_id,
            symbol: Rc::new(symbol.into()),
        }
    }
    fn lookup(&mut self, symbol: &str) -> Result<BindingKind, Error> {
        if self.definitions.contains(symbol) {
            return Ok(BindingKind::Module {
                module_id: self.module_id,
                symbol: Rc::new(symbol.into()),
            });
        }
        return Err(Error::UnboundIdentifier(symbol.into()));
    }
}
