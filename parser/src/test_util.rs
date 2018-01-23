#![cfg(test)]
use super::*;
use typed_arena::Arena;
use lexer::*;

pub fn with_parsed<'parse, F>(string: &'static str, f: F)
where
    F: FnOnce(Result),
{
    let lexed = lex(string);
    let arena = Arena::new();
    let parsed = parse_top(&lexed, &arena);
    f(parsed)
}
