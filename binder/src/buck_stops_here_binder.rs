use super::*;

pub struct BuckStopsHereBinder;

impl<'bound> Binder<'bound> for BuckStopsHereBinder {
    fn add_declaration(&mut self, _: &'bound str) -> BindingKind {
        panic!("add declaration on buck stops here");
    }

    fn lookup(&mut self, symbol: &str) -> Result<BindingKind, Error> {
        return Err(Error::UnboundIdentifier(symbol.into()));
    }
}
