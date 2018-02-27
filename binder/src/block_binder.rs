use super::*;

pub struct BlockBinder<'a, 'bound: 'a> {
    pub parent: &'a mut Binder<'bound>,
    pub definitions: HashMap<DeclarationKind<'bound>, DeclarationKind<'bound>>,
}

impl<'a, 'bound> Binder<'bound> for BlockBinder<'a, 'bound> {
    fn add_declaration(
        &mut self,
        symbol: DeclarationKind<'bound>,
        binding_state: &mut BindingState,
    ) -> BindingKind<'bound> {
        let new = match symbol.clone() {
            DeclarationKind::Generated(_, name) => {
                DeclarationKind::Generated(binding_state.gen_id(), name)
            }
            DeclarationKind::Named(name) => {
                DeclarationKind::Generated(binding_state.gen_id(), name)
            }
        };
        self.definitions.insert(symbol, new.clone());
        self.parent.add_declaration(new, binding_state)
    }

    fn lookup(&mut self, symbol: &DeclarationKind<'bound>) -> Result<BindingKind<'bound>, Error> {
        if let Some(resolved) = self.definitions.get(symbol) {
            self.parent.lookup(resolved)
        } else {
            self.parent.lookup(symbol)
        }
    }
}
