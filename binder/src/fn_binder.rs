use super::*;
use std::collections::HashMap;

pub struct FnBinder<'a, 'bound: 'a> {
    parent: &'a mut Binder<'bound>,
    locals: Vec<&'bound str>,
    arguments: &'bound [(&'bound str, &'bound Ast<'bound>)],
    upvars: HashMap<&'bound str, (BindingKind, u32)>,
}

impl<'a, 'bound> Binder<'bound> for FnBinder<'a, 'bound> {
    fn add_declaration(&mut self, symbol: &'bound str) -> BindingKind {
        let pos = self.locals.len();
        self.locals.push(symbol);
        BindingKind::FunctionLocal(pos as u32)
    }

    fn lookup(&mut self, symbol: &'bound str) -> Result<BindingKind, Error> {
        if let Some(pos) = self.arguments.iter().rposition(|&(l, _)| l == symbol) {
            return Ok(BindingKind::Argument(pos as u32));
        }
        if let Some(pos) = self.locals.iter().rposition(|l| &**l == symbol) {
            return Ok(BindingKind::FunctionLocal(pos as u32));
        }
        if let Some(&(_, p)) = self.upvars.get(symbol) {
            return Ok(BindingKind::Upvar(p));
        }
        if let Ok(bk) = self.parent.lookup(symbol) {
            let num = self.upvars.len() as u32;
            self.upvars.insert(symbol, (bk, num));
            return Ok(BindingKind::Upvar(num));
        }
        return Err(Error::UnboundIdentifier(symbol.into()));
    }
}

pub fn bind_function_decl<'bound>(
    parent: &mut Binder<'bound>,
    full_ast: &'bound Ast<'bound>,
    arena: &'bound Arena<Bound<'bound>>,

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
        };

        let body = arena.alloc(bind(arena, &mut binder, body)?);
        (binder.locals, binder.upvars, body)
    };

    return Ok(Bound::FunctionDecl {
        name,
        body,
        locals,
        upvars,
        ast: full_ast,
        params: params.to_vec(),
        location: parent.add_declaration(name),
    });
}
