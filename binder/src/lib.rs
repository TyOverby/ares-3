extern crate lexer;
extern crate parser;
extern crate typed_arena;

mod fn_binder;
mod module_binder;
mod block_binder;
mod buck_stops_here_binder;
#[cfg(test)]
mod test;

use std::rc::Rc;
use std::collections::{HashMap, HashSet};
use parser::Ast;
use typed_arena::Arena;

use module_binder::ModuleBinder;

#[derive(Debug)]
pub enum BindingKind<'bound> {
    FunctionLocal(u32),
    Argument(u32),
    Upvar(u32),
    Module {
        module_id: &'bound str,
        symbol: Rc<DeclarationKind<'bound>>,
    },
}

pub struct BindingState {
    gen_id: u64,
}

#[derive(Debug)]
pub enum Error {
    UnboundIdentifier(String),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum DeclarationKind<'bound> {
    Named(&'bound str),
    Generated(u64, &'bound str),
}

pub trait Binder<'bound> {
    fn add_declaration(
        &mut self,
        symbol: DeclarationKind<'bound>,
        &mut BindingState,
    ) -> BindingKind<'bound>;
    fn lookup(&mut self, symbol: &DeclarationKind<'bound>) -> Result<BindingKind<'bound>, Error>;
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
        binding_kind: BindingKind<'bound>,
    },
    DebugCall {
        ast: &'bound Ast<'bound>,
        arg: &'bound Bound<'bound>,
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
        params: Vec<(DeclarationKind<'bound>, &'bound Ast<'bound>)>,
        body: &'bound Bound<'bound>,
        locals: Vec<DeclarationKind<'bound>>,
        upvars: HashMap<DeclarationKind<'bound>, (BindingKind<'bound>, u32)>,
        ast: &'bound Ast<'bound>,
        location: BindingKind<'bound>,
    },
    VariableDecl {
        name: &'bound str,
        expression_ast: &'bound Ast<'bound>,
        expression: &'bound Bound<'bound>,
        location: BindingKind<'bound>,
    },
    FieldAccess {
        target_ast: &'bound Ast<'bound>,
        field_ast: &'bound Ast<'bound>,
        field_name: &'bound str,
        target: &'bound Bound<'bound>,
    },
    BlockExpr {
        statements: Vec<Bound<'bound>>,
        ast: &'bound Ast<'bound>,
        final_expression: &'bound Bound<'bound>,
    },
    Module {
        ast: &'bound Ast<'bound>,
        statements: Vec<Bound<'bound>>,
        binder: ModuleBinder<'bound>,
    },
}

impl BindingState {
    fn new() -> BindingState {
        BindingState { gen_id: 0 }
    }

    pub fn gen_id(&mut self) -> u64 {
        self.gen_id += 1;
        self.gen_id
    }
}

pub fn bind_top<'bound>(
    arena: &'bound Arena<Bound<'bound>>,
    ast: &'bound Ast<'bound>,
) -> Result<Bound<'bound>, Error> {
    let mut top_binder = buck_stops_here_binder::BuckStopsHereBinder;
    let mut binding_state = BindingState::new();
    bind(arena, &mut top_binder, &mut binding_state, ast)
}

fn bind<'bound>(
    arena: &'bound Arena<Bound<'bound>>,
    binder: &mut Binder<'bound>,
    binding_state: &mut BindingState,
    ast: &'bound Ast<'bound>,
) -> Result<Bound<'bound>, Error> {
    let bound = match ast {
        &Ast::Integer(_, value) => Bound::Integer { ast, value },
        &Ast::Float(_, value) => Bound::Float { ast, value },
        &Ast::Add(ast_left, ast_right) => Bound::Add {
            ast_left,
            ast_right,
            left: arena.alloc(bind(arena, binder, binding_state, ast_left)?),
            right: arena.alloc(bind(arena, binder, binding_state, ast_right)?),
        },
        &Ast::Sub(ast_left, ast_right) => Bound::Sub {
            ast_left,
            ast_right,
            left: arena.alloc(bind(arena, binder, binding_state, ast_left)?),
            right: arena.alloc(bind(arena, binder, binding_state, ast_right)?),
        },
        &Ast::Mul(ast_left, ast_right) => Bound::Mul {
            ast_left,
            ast_right,
            left: arena.alloc(bind(arena, binder, binding_state, ast_left)?),
            right: arena.alloc(bind(arena, binder, binding_state, ast_right)?),
        },
        &Ast::FieldAccess {
            target,
            field,
            field_name,
        } => Bound::FieldAccess {
            target_ast: target,
            field_ast: field,
            field_name,
            target: arena.alloc(bind(arena, binder, binding_state, target)?),
        },
        &Ast::Div(ast_left, ast_right) => Bound::Div {
            ast_left,
            ast_right,
            left: arena.alloc(bind(arena, binder, binding_state, ast_left)?),
            right: arena.alloc(bind(arena, binder, binding_state, ast_right)?),
        },
        &Ast::Pipeline(ast_left, ast_right) => Bound::Pipeline {
            ast_left,
            ast_right,
            left: arena.alloc(bind(arena, binder, binding_state, ast_left)?),
            right: arena.alloc(bind(arena, binder, binding_state, ast_right)?),
        },
        &Ast::Identifier(_, ident) => Bound::Identifier {
            ast,
            ident,
            binding_kind: binder.lookup(&DeclarationKind::Named(ident.into()))?,
        },
        &Ast::DebugCall(arg) => Bound::DebugCall {
            ast,
            arg: arena.alloc(bind(arena, binder, binding_state, arg)?),
        },
        &Ast::FunctionCall { target, ref args } => Bound::FunctionCall {
            ast,
            target: arena.alloc(bind(arena, binder, binding_state, target)?),
            args: args.iter()
                .map(|arg| bind(arena, binder, binding_state, arg))
                .collect::<Result<Vec<_>, _>>()?,
        },
        &Ast::BlockExpr {
            ref statements,
            ref final_expression,
        } => {
            let mut block_binder = block_binder::BlockBinder {
                parent: binder,
                definitions: HashMap::new(),
            };
            Bound::BlockExpr {
                statements: statements
                    .iter()
                    .map(|a| bind(arena, &mut block_binder, binding_state, a))
                    .collect::<Result<Vec<_>, _>>()?,
                final_expression: arena.alloc(bind(
                    arena,
                    &mut block_binder,
                    binding_state,
                    final_expression,
                )?),
                ast: ast,
            }
        }
        &Ast::Module {
            ref statements,
            module_id,
        } => {
            let mut module_binder = module_binder::ModuleBinder {
                module_id,
                definitions: HashSet::new(),
            };
            Bound::Module {
                ast,
                statements: statements
                    .iter()
                    .map(|stmt| bind(arena, &mut module_binder, binding_state, stmt))
                    .collect::<Result<Vec<_>, _>>()?,
                binder: module_binder,
            }
        }
        &Ast::VariableDecl {
            name,
            name_ast: _,
            expression,
        } => Bound::VariableDecl {
            name,
            expression_ast: expression,
            expression: arena.alloc(bind(arena, binder, binding_state, expression)?),
            location: binder.add_declaration(DeclarationKind::Named(name.into()), binding_state),
        },
        &Ast::FunctionDecl {
            name,
            ref params,
            body,
            ..
        } => fn_binder::bind_function_decl(binder, ast, arena, binding_state, name, params, body)?,
    };

    Ok(bound)
}
