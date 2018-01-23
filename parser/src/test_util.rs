#![cfg(test)]
#![allow(dead_code)]
use super::*;
use lexer::*;

pub fn with_parsed<'parse, F>(string: &'static str, f: F)
where
    F: FnOnce(Result),
{
    use typed_arena::Arena;
    let lexed = lex(string);
    let arena = Arena::new();
    let parsed = parse_top(&lexed, &arena);
    f(parsed)
}

pub fn with_specific_parsed<G, F>(string: &'static str, g: G, f: F)
where
    F: FnOnce(Result),
    G: for <'parse> Fn(&'parse [Token<'parse>], Arena<'parse>, &mut ParseCache<'parse>)
        -> Result<'parse>,
{
    let lexed = lex(string);
    let arena = ::typed_arena::Arena::new();
    let mut cache = HashMap::new();
    {
        let parsed = g(&lexed, &arena, &mut cache);
        f(parsed)
    }
}
