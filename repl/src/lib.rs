extern crate binder;
extern crate emit;
extern crate lexer;
extern crate parser;
extern crate typed_arena;
extern crate vm;

use parser::Ast;
use typed_arena::Arena;
use vm::value::Value;
use binder::{bind, BindingState, Bound, DeclarationKind, ModuleBinder};
use std::collections::HashSet;
use vm::vm::{Instruction, Vm};
use vm::value::{new_func, Function};
use lexer::{lex, remove_whitespace, Token};
use parser::{parse_expression, parse_module};
use std::collections::HashMap;

#[derive(Clone)]
pub struct StorableModuleBinder {
    pub name: String,
    pub definitions: HashSet<String>,
}

enum ReplParseResult<'parse> {
    Expression(&'parse Ast<'parse>),
    Statement(&'parse Ast<'parse>),
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
}

fn repl_parse_expression<'parse>(
    lexed: &'parse [Token<'parse>],
    arena: &'parse Arena<Ast<'parse>>,
) -> Result<&'parse Ast<'parse>, parser::ParseError<'parse>> {
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
) -> Result<&'parse Ast<'parse>, parser::ParseError<'parse>> {
    let mut cache = HashMap::new();

    let parsed = parse_module(&lexed, "repl-module", &arena, &mut cache);

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
        (Err(e), _) => ReplParseResult::Error(e),
    }
}

pub fn run(
    program: &str,
    vm: &mut Vm,
    past_work: StorableModuleBinder,
) -> Result<(Value, StorableModuleBinder), String> {
    use binder::bind_top;
    use emit::emit_top;

    let mut lexed = lex(program);
    remove_whitespace(&mut lexed);

    let parse_arena = Arena::new();
    let bind_arena = Arena::new();

    let parsed = do_parse(&lexed, &parse_arena);
    let (emitted, new_mod_binder) = match parsed {
        ReplParseResult::Expression(e) => {
            let mut module_binder = past_work.to_module_binder();
            let mut binder_state = BindingState { gen_id: 0 };
            let bound = bind(&bind_arena, &mut module_binder, &mut binder_state, e).unwrap();

            let mut instrs = vec![];
            emit::emit(&bound, &mut instrs, None);
            instrs.push(Instruction::Ret);
            let function = new_func(Function {
                name: None,
                instructions: instrs,
                upvars: vec![],

                args_count: 0,
                upvars_count: 0,
                locals_count: 0,
            });
            (Value::Function(function), past_work.clone())
        }
        ReplParseResult::Statement(s) => {
            let mut module_binder = past_work.to_module_binder();
            let mut binder_state = BindingState { gen_id: 0 };
            let bound = bind(&bind_arena, &mut module_binder, &mut binder_state, &s).unwrap();
            let emitted = emit_top(&bound);
            if let Bound::Module { binder, .. } = bound {
                (emitted, StorableModuleBinder::from_module_binder(&binder))
            } else {
                panic!("non-module bound somehow");
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

    let value = match vm.run_function(f) {
        Ok(v) => v,
        Err(e) => return Err(format!("{:?}", e)),
    };

    Ok((value, new_mod_binder))
}
