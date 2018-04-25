use super::{translate, ContAst, ContAstPtr, IdGet, PrimOpKind, Terminal};
use difference::Changeset;
use lexer::{lex, remove_whitespace};
use parser::parse_expression;
use std::collections::HashMap;
use typed_arena::Arena;

fn with_parsed_expression_cont<F>(string: &'static str, f: F)
where
    F: FnOnce(ContAstPtr),
{
    let id_builder = IdGet::new();
    let mut lexed = lex(string);
    remove_whitespace(&mut lexed);
    let arena = Arena::new();
    let new_arena = Arena::new();
    let mut cache = HashMap::new();
    let parsed = parse_expression(&lexed, &arena, &mut cache).unwrap().0;
    println!("{:#?}", parsed);
    let parsed_cont =
        translate(parsed, Box::new(|t| generate_terminal(&new_arena, t)), &id_builder, &new_arena);
    f(parsed_cont)
}

fn generate_terminal<'a>(arena: &'a Arena<ContAst<'a>>, terminal: Terminal<'a>) -> ContAstPtr<'a> {
    arena.alloc(ContAst::Primop {
        op: PrimOpKind::Term,
        terminals: vec![terminal],
        exports: vec![],
        continuations: vec![],
    })
}

fn compile_eq(program: &'static str, expected: &str) {
    fn str_rep_eq(actual: String, expected: &str) {
        if actual.trim() != expected.trim() {
            panic!("\n{}", Changeset::new(actual.trim(), expected.trim(), "\n"));
        }
    }

    with_parsed_expression_cont(program, |r| {
        str_rep_eq(format!("{:?}", r), expected);
    });
}
#[test]
fn simple_identifier() {
    compile_eq("c", "Term([c]) -> ([]) =>");
}

#[test]
fn simple_number() {
    compile_eq("1", "Term([1]) -> ([]) =>");
}

#[test]
fn no_arg_call() {
    compile_eq(
        "a()",
        r#"
fix fn id_0([id_1]) =>
    Term([id_1]) -> ([]) =>
    continue with:
        call a([]) -> id_0
    "#,
    );
}

#[test]
fn no_arg_call_in_expression() {
    compile_eq(
        "a() + b() * c()",
        r#"
fix fn id_1([id_2]) =>
    fix fn id_4([id_5]) =>
        fix fn id_6([id_7]) =>
            Mul([id_5, id_7]) -> ([id_3]) =>
                Add([id_2, id_3]) -> ([id_0]) =>
                    Term([id_0]) -> ([]) =>
            continue with:
                call c([]) -> id_6
        continue with:
            call b([]) -> id_4
    continue with:
        call a([]) -> id_1
    "#,
    );
}

#[test]
fn one_arg_int_call() {
    compile_eq(
        "a(5)",
        r#"
fix fn id_0([id_1]) =>
    Term([id_1]) -> ([]) =>
    continue with:
        call a([5]) -> id_0
    "#,
    );
}

#[test]
fn one_arg_expression_call() {
    compile_eq(
        "a(5 + 10)",
        r#"
fix fn id_0([id_1]) =>
    Term([id_1]) -> ([]) =>
    continue with:
        Add([5, 10]) -> ([id_2]) =>
            call a([id_2]) -> id_0
    "#,
    );
}

#[test]
fn appel_13() {
    compile_eq(
        "(a + 1) * (c + 3)",
        r#"
Add([a, 1]) -> ([id_1]) =>
    Add([c, 3]) -> ([id_2]) =>
        Mul([id_1, id_2]) -> ([id_0]) =>
            Term([id_0]) -> ([]) =>"#,
    );
}
#[test]
fn appel_13_mod() {
    compile_eq(
        "a + b * c",
        r#"
Mul([b, c]) -> ([id_1]) =>
    Add([a, id_1]) -> ([id_0]) =>
        Term([id_0]) -> ([]) =>"#,
    );
}
