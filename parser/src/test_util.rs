#![cfg(test)]
#![allow(dead_code)]
use super::*;
use lexer::*;

pub fn with_parsed_expression<'parse, F>(string: &'static str, f: F)
where
    F: FnOnce(Result),
{
    let mut arena = copy_arena::Arena::new();
    let mut alloc = arena.allocator();

    let lexed = lex(string, &mut alloc);
    let lexed = remove_whitespace(&lexed, &mut alloc);
    let parsed = parse_expression(&lexed, &mut alloc);
    f(parsed)
}

pub fn with_parsed_statement<'a, F>(string: &'static str, f: F)
where
    F: FnOnce(Result),
{
    let mut arena = copy_arena::Arena::new();
    let mut alloc = arena.allocator();

    let lexed = lex(string, &mut alloc);
    let lexed = remove_whitespace(&lexed, &mut alloc);
    let parsed = parse_statement(&lexed, &mut alloc);
    f(parsed)
}

pub fn with_parsed_module<'parse, F>(string: &'static str, module_id: &'static str, f: F)
where
    F: FnOnce(Result),
{
    let mut arena = copy_arena::Arena::new();
    let mut alloc = arena.allocator();

    let lexed = lex(string, &mut alloc);
    let lexed = remove_whitespace(&lexed, &mut alloc);
    let parsed = parse_module(&lexed, module_id, &mut alloc);
    f(parsed)
}
