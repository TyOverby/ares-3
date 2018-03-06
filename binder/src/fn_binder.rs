use super::*;
use std::collections::HashMap;

pub struct FnBinder<'a, 'bound: 'a> {
    parent: &'a mut Binder<'bound>,
    locals: Vec<DeclarationKind<'bound>>,
    arguments: &'bound [(&'bound str, &'bound Ast<'bound>)],
    upvars: HashMap<DeclarationKind<'bound>, (BindingKind<'bound>, u32)>,
    name: &'bound str,
}

impl<'a, 'bound> Binder<'bound> for FnBinder<'a, 'bound> {
    fn add_declaration(&mut self, symbol: DeclarationKind<'bound>, _: &mut BindingState) -> BindingKind<'bound> {
        let pos = self.locals.len();
        self.locals.push(symbol);
        BindingKind::FunctionLocal(pos as u32)
    }

    fn lookup(&mut self, symbol: &DeclarationKind<'bound>) -> Result<BindingKind<'bound>, Error> {
        if let Some(pos) = self.arguments
            .iter()
            .rposition(|&(l, _)| &DeclarationKind::Named(l) == symbol)
        {
            return Ok(BindingKind::Argument(pos as u32));
        }
        if let Some(pos) = self.locals.iter().rposition(|l| l == symbol) {
            return Ok(BindingKind::FunctionLocal(pos as u32));
        }
        if let Some(&(_, p)) = self.upvars.get(&symbol) {
            return Ok(BindingKind::Upvar(p));
        }

        if &DeclarationKind::Named(self.name) == symbol {
            return Ok(BindingKind::CurrentFunction);
        }

        if let Ok(bk) = self.parent.lookup(symbol) {
            let num = self.upvars.len() as u32;
            self.upvars.insert(symbol.clone(), (bk, num));
            return Ok(BindingKind::Upvar(num));
        }

        match symbol {
            &DeclarationKind::Named(s) => Err(Error::UnboundIdentifier(s.into())),
            &DeclarationKind::Generated(_, s) => Err(Error::UnboundIdentifier(s.into())),
        }
    }
}

pub fn bind_function_decl<'bound>(
    parent: &mut Binder<'bound>,
    full_ast: &'bound Ast<'bound>,
    arena: &'bound Arena<Bound<'bound>>,
    binding_state: &mut BindingState,

    name: &'bound str,
    params: &'bound [(&'bound str, &'bound Ast<'bound>)],
    body: &'bound Ast<'bound>,
) -> Result<Bound<'bound>, Error> {
    let (locals, upvars, body) = {
        let mut binder = FnBinder {
            parent: parent,
            locals: vec![],
            arguments: &params,
            upvars: HashMap::new(),
            name: name,
        };

        let body = arena.alloc(bind(arena, &mut binder, binding_state, body)?);
        (binder.locals, binder.upvars, body)
    };

    return Ok(Bound::FunctionDecl {
        name,
        body,
        locals,
        upvars,
        ast: full_ast,
        params: params
            .into_iter()
            .map(|&(n, ast)| (DeclarationKind::Named(n.into()), ast))
            .collect(),
        location: parent.add_declaration(DeclarationKind::Named(name.into()), binding_state),
    });
}
