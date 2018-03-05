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

#[test]
fn emit_module_with_expression_statement() {
    let instrs = emit_module("5;");
    assert_eq!(&instrs, &[Push(Value::Integer(5)), Pop, MapEmpty, Ret]);
}

#[test]
fn emit_module_with_variable_declaration() {
    let instrs = emit_module("let x = 5;");
    assert_eq!(
        &instrs,
        &[
            Push(Value::Integer(5)),
            Push(Value::symbol("my_module")),
            Push(Value::symbol("x")),
            ModuleAdd,
            MapEmpty,
            Ret
        ]
    );
}

#[test]
fn emit_module_with_variable_declaration_and_variable_access() {
    let instrs = emit_module("let x = 5; x;");
    assert_eq!(
        &instrs,
        &[
            Push(Value::Integer(5)),
            Push(Value::symbol("my_module")),
            Push(Value::symbol("x")),
            ModuleAdd,
            Push(Value::symbol("my_module")),
            Push(Value::symbol("x")),
            ModuleGet,
            Pop,
            MapEmpty,
            Ret
        ]
    );
}

#[test]
fn emit_fn_delcaration() {
    let instrs = emit_module("let id(x) = x;");
    assert_eq!(
        &instrs[..],
        &[
            Push(Value::Function(new_func(Function {
                instructions: vec![GetFromStackPosition(0)],
                name: Some("id".into()),
                arg_count: 1,
            }))),
            Push(Value::symbol("my_module")),
            Push(Value::symbol("id")),
            ModuleAdd,
            MapEmpty,
            Ret
        ]
    );
}

#[test]
fn emit_debug_statement() {
    let instrs = emit_module("debug(10);");
    assert_eq!(
        &instrs[..],
        &[Push(Value::Integer(10)), Debug, MapEmpty, Ret]
    );
}
