#![cfg(test)]
#![allow(dead_code)]
use super::*;
use lexer::*;

pub fn with_parsed_expression<'parse, F>(string: &'static str, f: F)
where
    F: FnOnce(Result),
{
    use typed_arena::Arena;
    let mut lexed = lex(string);
    remove_whitespace(&mut lexed);
    let arena = Arena::new();
    let mut cache = HashMap::new();
    let parsed = parse_expression(&lexed, &arena, &mut cache);
    f(parsed)
}

pub fn with_parsed_statement<'parse, F>(string: &'static str, f: F)
where
    F: FnOnce(Result),
{
    use typed_arena::Arena;
    let mut lexed = lex(string);
    remove_whitespace(&mut lexed);
    let arena = Arena::new();
    let mut cache = HashMap::new();
    let parsed = parse_statement(&lexed, &arena, &mut cache);
    f(parsed)
}

pub fn with_parsed_module<'parse, F>(string: &'static str, module_id: &'static str, f: F)
where
    F: FnOnce(Result),
{
    use typed_arena::Arena;
    let mut lexed = lex(string);
    remove_whitespace(&mut lexed);
    let arena = Arena::new();
    let mut cache = HashMap::new();
    let parsed = parse_module(&lexed, module_id, &arena, &mut cache);
    f(parsed)
}

pub fn with_specific_parsed<G, F>(string: &'static str, g: G, f: F)
where
    F: FnOnce(Result),
    G: for<'parse> Fn(&'parse [Token<'parse>], Arena<'parse>, &mut ParseCache<'parse>)
        -> Result<'parse>,
{
    let mut lexed = lex(string);
    remove_whitespace(&mut lexed);
    let arena = ::typed_arena::Arena::new();
    let mut cache = HashMap::new();
    {
        let parsed = g(&lexed, &arena, &mut cache);
        f(parsed)
    }
}
