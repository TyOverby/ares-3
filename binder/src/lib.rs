extern crate lexer;
extern crate parser;
extern crate typed_arena;

mod fn_binder;
mod module_binder;
#[cfg(test)]
mod test;

use std::rc::Rc;
use std::collections::{HashMap, HashSet};
use parser::Ast;
use typed_arena::Arena;

#[derive(Debug)]
pub enum BindingKind {
    FunctionLocal(u32),
    Argument(u32),
    Upvar(u32),
    Module { module_id: u32, symbol: Rc<String> },
}

#[derive(Debug)]
pub enum Error {
    UnboundIdentifier(String),
}

pub trait Binder<'bound> {
    fn add_declaration(&mut self, symbol: &'bound str) -> BindingKind;
    fn lookup(&mut self, symbol: &'bound str) -> Result<BindingKind, Error>;
}

#[derive(Debug)]
pub enum Bound<'bound> {
    Integer {
        ast: &'bound Ast<'bound>,
        value: i64,
    },
    Float {
        ast: &'bound Ast<'bound>,
        value: f64,
    },
    Identifier {
        ast: &'bound Ast<'bound>,
        ident: &'bound str,
        binding_kind: BindingKind,
    },
    FunctionCall {
        ast: &'bound Ast<'bound>,
        target: &'bound Bound<'bound>,
        args: Vec<Bound<'bound>>,
    },
    Pipeline {
        ast_left: &'bound Ast<'bound>,
        ast_right: &'bound Ast<'bound>,
        left: &'bound Bound<'bound>,
        right: &'bound Bound<'bound>,
    },
    Add {
        ast_left: &'bound Ast<'bound>,
        ast_right: &'bound Ast<'bound>,
        left: &'bound Bound<'bound>,
        right: &'bound Bound<'bound>,
    },
    Sub {
        ast_left: &'bound Ast<'bound>,
        ast_right: &'bound Ast<'bound>,
        left: &'bound Bound<'bound>,
        right: &'bound Bound<'bound>,
    },
    Div {
        ast_left: &'bound Ast<'bound>,
        ast_right: &'bound Ast<'bound>,
        left: &'bound Bound<'bound>,
        right: &'bound Bound<'bound>,
    },
    Mul {
        ast_left: &'bound Ast<'bound>,
        ast_right: &'bound Ast<'bound>,
        left: &'bound Bound<'bound>,
        right: &'bound Bound<'bound>,
    },
    FunctionDecl {
        name: &'bound str,
        params: Vec<(&'bound str, &'bound Ast<'bound>)>,
        body: &'bound Bound<'bound>,
        locals: Vec<&'bound str>,
        upvars: HashMap<&'bound str, (BindingKind, u32)>,
        ast: &'bound Ast<'bound>,
        location: BindingKind,
    },
    VariableDecl {
        name: &'bound str,
        expression_ast: &'bound Ast<'bound>,
        expression: &'bound Bound<'bound>,
        location: BindingKind,
    },
    FieldAccess {
        target_ast: &'bound Ast<'bound>,
        field_ast: &'bound Ast<'bound>,
        field_name: &'bound str,
        target: &'bound Bound<'bound>,
    },
    BlockExpr {
        statements: Vec<(&'bound Ast<'bound>, &'bound Bound<'bound>)>,
        final_expression_ast: &'bound Ast<'bound>,
        final_expression: &'bound Bound<'bound>,
    },
}

pub fn bind_top<'bound>(
    arena: &'bound Arena<Bound<'bound>>,
    module_id: u32,
    ast: &'bound Ast<'bound>,
) -> Result<Bound<'bound>, Error> {
    let mut module_binder = module_binder::ModuleBinder {
        module_id,
        definitions: HashSet::new(),
    };

    bind(arena, &mut module_binder, ast)
}

fn bind<'bound>(
    arena: &'bound Arena<Bound<'bound>>,
    binder: &mut Binder<'bound>,
    ast: &'bound Ast<'bound>,
) -> Result<Bound<'bound>, Error> {
    let bound = match ast {
        &Ast::Integer(_, value) => Bound::Integer { ast, value },
        &Ast::Float(_, value) => Bound::Float { ast, value },
        &Ast::Add(ast_left, ast_right) => Bound::Add {
            ast_left,
            ast_right,
            left: arena.alloc(bind(arena, binder, ast_left)?),
            right: arena.alloc(bind(arena, binder, ast_right)?),
        },
        &Ast::Sub(ast_left, ast_right) => Bound::Sub {
            ast_left,
            ast_right,
            left: arena.alloc(bind(arena, binder, ast_left)?),
            right: arena.alloc(bind(arena, binder, ast_right)?),
        },
        &Ast::Mul(ast_left, ast_right) => Bound::Mul {
            ast_left,
            ast_right,
            left: arena.alloc(bind(arena, binder, ast_left)?),
            right: arena.alloc(bind(arena, binder, ast_right)?),
        },
        &Ast::FieldAccess {
            target,
            field,
            field_name,
        } => Bound::FieldAccess {
            target_ast: target,
            field_ast: field,
            field_name,
            target: arena.alloc(bind(arena, binder, target)?),
        },
        &Ast::Div(ast_left, ast_right) => Bound::Div {
            ast_left,
            ast_right,
            left: arena.alloc(bind(arena, binder, ast_left)?),
            right: arena.alloc(bind(arena, binder, ast_right)?),
        },
        &Ast::Pipeline(ast_left, ast_right) => Bound::Pipeline {
            ast_left,
            ast_right,
            left: arena.alloc(bind(arena, binder, ast_left)?),
            right: arena.alloc(bind(arena, binder, ast_right)?),
        },
        &Ast::Identifier(_, ident) => Bound::Identifier {
            ast,
            ident,
            binding_kind: binder.lookup(ident)?,
        },
        &Ast::FunctionCall { target, ref args } => Bound::FunctionCall {
            ast,
            target: arena.alloc(bind(arena, binder, target)?),
            args: args.iter()
                .map(|arg| bind(arena, binder, arg))
                .collect::<Result<Vec<_>, _>>()?,
        },
        &Ast::BlockExpr { .. } => unimplemented!(),
        &Ast::VariableDecl {
            name,
            name_ast: _,
            expression,
        } => Bound::VariableDecl {
            name,
            expression_ast: expression,
            expression: arena.alloc(bind(arena, binder, expression)?),
            location: binder.add_declaration(name),
        },
        &Ast::FunctionDecl {
            name,
            ref params,
            body,
            ..
        } => fn_binder::bind_function_decl(binder, ast, arena, name, params, body)?,
    };

    Ok(bound)
}
