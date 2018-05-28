extern crate binder;
extern crate emit;
extern crate lexer;
extern crate parser;
extern crate typed_arena;
extern crate vm;

use binder::{bind, BindingState, Bound, DeclarationKind, ModuleBinder};
use lexer::{lex, remove_whitespace, Token};
use parser::Ast;
use parser::{parse_expression, parse_statement};
use std::collections::HashMap;
use std::collections::HashSet;
use typed_arena::Arena;
use vm::value::Value;
use vm::value::{new_func, Function};
use vm::vm::Vm;

#[derive(Clone, Debug)]
pub struct StorableModuleBinder {
    pub name: String,
    pub definitions: HashSet<String>,
}

pub enum ReplOutKind {
    Expression(Value),
    Statement(Value),
}

enum ReplParseResult<'parse> {
    Expression(AstPtr<'parse>),
    Statement(AstPtr<'parse>),
    KeepTrying,
    Error(parser::ParseError<'parse>),
}

impl StorableModuleBinder {
    fn to_module_binder<'a>(&'a self) -> ModuleBinder<'a> {
        ModuleBinder {
            module_id: &self.name,
            definitions: self.definitions
                .iter()
                .map(|s| DeclarationKind::Named(&s))
                .collect(),
        }
    }
    fn from_module_binder(mb: &ModuleBinder) -> StorableModuleBinder {
        StorableModuleBinder {
            name: mb.module_id.into(),
            definitions: mb.definitions
                .iter()
                .filter_map(|s| match s {
                    &DeclarationKind::Named(s) => Some(s.to_string()),
                    _ => None,
                })
                .collect(),
        }
    }
    fn add_additional(&mut self, mb: &StorableModuleBinder) {
        for def in &mb.definitions {
            self.definitions.insert(def.clone());
        }
    }
}

fn repl_parse_expression<'parse>(
    lexed: &'parse [Token<'parse>],
    arena: &'parse Arena<Ast<'parse>>,
) -> Result<AstPtr<'parse>, parser::ParseError<'parse>> {
    let mut cache = HashMap::new();

    let parsed = parse_expression(&lexed, &arena, &mut cache);

    match parsed {
        Ok((ast, _)) => Ok(ast),
        Err((e, _)) => Err(e),
    }
}

fn repl_parse_statement<'parse>(
    lexed: &'parse [Token<'parse>],
    arena: &'parse Arena<Ast<'parse>>,
) -> Result<AstPtr<'parse>, parser::ParseError<'parse>> {
    let mut cache = HashMap::new();

    let parsed = parse_statement(&lexed, &arena, &mut cache);

    match parsed {
        Ok((ast, _)) => Ok(ast),
        Err((e, _)) => Err(e),
    }
}

fn do_parse<'parse>(
    lexed: &'parse [Token<'parse>],
    arena: &'parse Arena<Ast<'parse>>,
) -> ReplParseResult<'parse> {
    match (
        repl_parse_expression(lexed, arena),
        repl_parse_statement(lexed, arena),
    ) {
        (Ok(e), _) => ReplParseResult::Expression(e),
        (_, Ok(s)) => ReplParseResult::Statement(s),
        (_, Err(e)) => ReplParseResult::Error(e),
    }
}

pub fn run(
    program: &str,
    vm: &mut Vm,
    past_work: StorableModuleBinder,
) -> Result<(ReplOutKind, StorableModuleBinder), String> {
    use emit::emit_top;

    let mut lexed = lex(program);
    remove_whitespace(&mut lexed);

    let parse_arena = Arena::new();
    let bind_arena = Arena::new();

    let parsed = do_parse(&lexed, &parse_arena);
    let (emitted, new_mod_binder, is_expression) = match parsed {
        ReplParseResult::Expression(e) => {
            let mut module_binder = past_work.to_module_binder();
            let mut binder_state = BindingState { gen_id: 0 };
            let bound = match bind(&bind_arena, &mut module_binder, &mut binder_state, e) {
                Ok(b) => b,
                Err(e) => return Err(format!("{:?}", e)),
            };

            let mut instrs = vec![];
            emit::emit(&bound, &mut instrs, None);
            panic!(); //instrs.push(Instruction::Ret);
            let function = new_func(Function {
                name: None,
                instructions: instrs,
                upvars: vec![],

                args_count: 0,
                upvars_count: 0,
                locals_count: 0,
            });
            (Value::Function(function), past_work.clone(), true)
        }
        ReplParseResult::Statement(s) => {
            let mut module_binder = past_work.to_module_binder();
            let mut binder_state = BindingState { gen_id: 0 };
            let bound = match bind(&bind_arena, &mut module_binder, &mut binder_state, &s) {
                Ok(b) => b,
                Err(e) => return Err(format!("{:?}", e)),
            };
            let bound = bind_arena.alloc(Bound::Module {
                ast: s,
                statements: vec![bound],
                binder: module_binder,
            }) as &_;
            let emitted = emit_top(&bound);
            if let &Bound::Module { ref binder, .. } = bound {
                let mut new = StorableModuleBinder::from_module_binder(&binder);
                new.add_additional(&past_work);
                (emitted, new, false)
            } else {
                unreachable!();
            }
        }
        ReplParseResult::Error(e) => {
            return Err(format!("{:?}", e));
        }
        ReplParseResult::KeepTrying => {
            panic!();
        }
    };

    let f = emitted.into_function().unwrap();

    let value = match (vm.run_function(f), is_expression) {
        (Ok(v), true) => ReplOutKind::Expression(v),
        (Ok(v), false) => ReplOutKind::Statement(v),
        (Err(e), _) => return Err(format!("{:?}", e)),
    };

    Ok((value, new_mod_binder))
}
