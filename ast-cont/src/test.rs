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
    let parsed_cont = translate(
        parsed,
        Box::new(|t| generate_terminal(&new_arena, t)),
        &id_builder,
        &new_arena,
    );
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

use PrimOpKind::*;
use Ident::*;
use Terminal::*;
use ContAst::*;

#[test]
fn appel_13() {
    with_parsed_expression_cont("(a + 1) * (c + 3)", |r| {
        assert_eq!(
            r,
            &Primop {
                op: Add,
                terminals: vec![Ident(Identifier("a")), Integer(1)],
                exports: vec![Phantom(1)],
                continuations: vec![
                    &Primop {
                        op: Add,
                        terminals: vec![Ident(Identifier("c")), Integer(3)],
                        exports: vec![Phantom(2)],
                        continuations: vec![
                            &Primop {
                                op: Mul,
                                terminals: vec![Ident(Phantom(1)), Ident(Phantom(2))],
                                exports: vec![Phantom(0)],
                                continuations: vec![
                                    &Primop {
                                        op: Term,
                                        terminals: vec![Ident(Phantom(0))],
                                        exports: vec![],
                                        continuations: vec![]
                                    }
                                ]
                            }
                        ]
                    }
                ]
            }
        );
    });
}

#[test]
fn appel_13_mod() {
    with_parsed_expression_cont("a + b * c", |r| {
        panic!("{:#?}", r);
    });
}
