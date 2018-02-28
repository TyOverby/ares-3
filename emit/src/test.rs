use ::*;
use parser::*;
use binder::*;
use lexer::*;

use vm::vm::Instruction::*;

fn emit_module(input: &str) -> Vec<Instruction> {
    use std::collections::HashMap;
    use typed_arena::Arena;
    use lexer::lex;

    let mut lexed = lex(input);
    let parse_arena = Arena::new();
    let bind_arena = Arena::new();

    remove_whitespace(&mut lexed);
    let mut cache = HashMap::new();
    let parsed = parse_module(&lexed, "my_module", &parse_arena, &mut cache).unwrap();
    let bound = bind_top(&bind_arena, parsed.0).unwrap();
    let emitted = emit_top(&bound);
    let f = emitted.into_function().unwrap().function;
    let &Function {
        ref instructions, ..
    } = &*f.borrow();
    instructions.clone()
}

fn remove_whitespace(tokens: &mut Vec<Token>) {
    tokens.retain(|token| {
        if let TokenKind::Whitespace(_) = token.kind {
            false
        } else {
            true
        }
    })
}

#[test]
fn emit_module_with_expression_statement() {
    let instrs = emit_module("5;");
    assert_eq!(instrs, vec![Push(Value::Integer(5)), Pop]);
}

#[test]
fn emit_module_with_variable_declaration() {
    let instrs = emit_module("let x = 5;");
    assert_eq!(
        instrs,
        vec![
            Push(Value::Integer(5)),
            Push(Value::Symbol(Symbol("my_module".into()))),
            Push(Value::Symbol(Symbol("x".into()))),
            ModuleAdd,
        ]
    );
}

#[test]
fn emit_module_with_variable_declaration_and_variable_access() {
    let instrs = emit_module("let x = 5; x;");
    assert_eq!(
        instrs,
        vec![
            Push(Value::Integer(5)),
            Push(Value::Symbol(Symbol("my_module".into()))),
            Push(Value::Symbol(Symbol("x".into()))),
            ModuleAdd,
            Push(Value::Symbol(Symbol("my_module".into()))),
            Push(Value::Symbol(Symbol("x".into()))),
            ModuleGet,
            Pop,
        ]
    );
}
