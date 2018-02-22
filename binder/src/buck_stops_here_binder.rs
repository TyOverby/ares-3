use super::*;

pub struct BuckStopsHereBinder;

impl<'bound> Binder<'bound> for BuckStopsHereBinder {
    fn add_declaration(&mut self, _: DeclarationKind, _: &mut BindingState) -> BindingKind {
        panic!("add declaration on buck stops here");
    }

    fn lookup(&mut self, symbol: &DeclarationKind) -> Result<BindingKind, Error> {
        match symbol {
            &DeclarationKind::Named(ref s) => Err(Error::UnboundIdentifier(s.clone())),
            &DeclarationKind::Generated(_, ref s) => Err(Error::UnboundIdentifier(s.clone())),
        }
    }
}
