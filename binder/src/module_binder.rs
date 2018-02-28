use super::*;
use std::collections::HashSet;

#[derive(Debug)]
pub struct ModuleBinder<'bound> {
    pub module_id: &'bound str,
    pub definitions: HashSet<DeclarationKind<'bound>>,
}

impl<'bound> Binder<'bound> for ModuleBinder<'bound> {
    fn add_declaration(
        &mut self,
        symbol: DeclarationKind<'bound>,
        _: &mut BindingState,
    ) -> BindingKind<'bound> {
        self.definitions.insert(symbol.clone());
        BindingKind::Module {
            module_id: self.module_id,
            symbol: Rc::new(symbol),
        }
    }

    fn lookup(&mut self, symbol: &DeclarationKind<'bound>) -> Result<BindingKind<'bound>, Error> {
        if self.definitions.contains(symbol) {
            return Ok(BindingKind::Module {
                module_id: self.module_id,
                symbol: Rc::new(symbol.clone()),
            });
        }

        match symbol {
            &DeclarationKind::Named(s) => Err(Error::UnboundIdentifier(s.into())),
            &DeclarationKind::Generated(_, s) => Err(Error::UnboundIdentifier(s.into())),
        }
    }
}
