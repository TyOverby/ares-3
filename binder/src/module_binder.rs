use super::*;
use std::collections::HashSet;

#[derive(Debug)]
pub struct ModuleBinder {
    pub module_id: u32,
    pub definitions: HashSet<DeclarationKind>,
}

impl<'bound> Binder<'bound> for ModuleBinder {
    fn add_declaration(&mut self, symbol: DeclarationKind, _: &mut BindingState) -> BindingKind {
        self.definitions.insert(symbol.clone());
        BindingKind::Module {
            module_id: self.module_id,
            symbol: Rc::new(symbol),
        }
    }
    fn lookup(&mut self, symbol: &DeclarationKind) -> Result<BindingKind, Error> {
        if self.definitions.contains(symbol) {
            return Ok(BindingKind::Module {
                module_id: self.module_id,
                symbol: Rc::new(symbol.clone()),
            });
        }

        match symbol {
            &DeclarationKind::Named(ref s) => Err(Error::UnboundIdentifier(s.clone())),
            &DeclarationKind::Generated(_, ref s) => Err(Error::UnboundIdentifier(s.clone())),
        }
    }
}
