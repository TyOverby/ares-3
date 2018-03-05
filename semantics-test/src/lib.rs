extern crate binder;
extern crate emit;
extern crate lexer;
extern crate parser;
extern crate typed_arena;
extern crate vm;

use vm::value::Value;

#[allow(dead_code)]
fn run(program: &str) -> Vec<Value> {
    use std::collections::HashMap;
    use typed_arena::Arena;
    use lexer::{lex, remove_whitespace};
    use parser::parse_module;
    use binder::bind_top;
    use emit::emit_top;

    let mut lexed = lex(program);
    let parse_arena = Arena::new();
    let bind_arena = Arena::new();

    remove_whitespace(&mut lexed);
    let mut cache = HashMap::new();
    let parsed = parse_module(&lexed, "my_module", &parse_arena, &mut cache).unwrap();
    let bound = bind_top(&bind_arena, parsed.0).unwrap();
    let emitted = emit_top(&bound);
    let f = emitted.into_function().unwrap();

    let mut vm = vm::vm::Vm::new(f);
    vm.run().unwrap();

    vm.debug_values
}

#[test]
fn test_basic_debug() {
    let out = run("debug(10);");
    assert_eq!(out, vec![Value::Integer(10)]);
}

#[test]
fn test_multiple_debug() {
    let out = run("debug(10); debug(20);");
    assert_eq!(out, vec![Value::Integer(10), Value::Integer(20)]);
}
