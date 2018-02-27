use super::*;

pub struct BuckStopsHereBinder;

impl<'bound> Binder<'bound> for BuckStopsHereBinder {
    fn add_declaration(
        &mut self,
        _: DeclarationKind<'bound>,
        _: &mut BindingState,
    ) -> BindingKind<'bound> {
        panic!("add declaration on buck stops here");
    }

    fn lookup(&mut self, symbol: &DeclarationKind<'bound>) -> Result<BindingKind<'bound>, Error> {
        match symbol {
            &DeclarationKind::Named(s) => Err(Error::UnboundIdentifier(s.into())),
            &DeclarationKind::Generated(_, s) => Err(Error::UnboundIdentifier(s.into())),
        }
    }
}
