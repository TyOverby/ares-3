use super::{translate, ContAst, ContAstPtr, IdGet, PrimOpKind, Terminal};
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

use difference::Changeset;
use ContAst::*;
use Ident::*;
use PrimOpKind::*;
use Terminal::*;

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
fn appel_13() {
    compile_eq( "(a + 1) * (c + 3)",
        r#"
Add([a, 1]) -> ([id_1]) =>
    Add([c, 3]) -> ([id_2]) =>
        Mul([id_1, id_2]) -> ([id_0]) =>
            Term([id_0]) -> ([]) =>"#,
    );
}
#[test]
fn appel_13_mod() {
    compile_eq("a + b * c",
    r#"
Mul([b, c]) -> ([id_1]) =>
    Add([a, id_1]) -> ([id_0]) =>
        Term([id_0]) -> ([]) =>"#);
}
