extern crate lexer;
extern crate typed_arena;

#[macro_use]
mod util;
mod test_util;
mod parts;

use std::collections::HashMap;
use lexer::{Token, TokenKind};
use parts::*;

type Arena<'lex, 'parse> = &'parse typed_arena::Arena<Ast<'lex, 'parse>>;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum CacheKey {
    Function,
}

type Result<'lex, 'parse> = std::result::Result<
    (&'parse Ast<'lex, 'parse>, &'lex [Token<'lex>]),
    (ParseError<'lex>, &'lex [Token<'lex>]),
>;

#[derive(Clone, Debug)]
pub enum ParseError<'lex> {
    UnexpectedToken {
        found: &'lex Token<'lex>,
        expected: &'static str,
    },
    Working,
    EndOfFileReached,
}

pub enum CacheState<'lex: 'parse, 'parse> {
    Working,
    Done((&'parse Ast<'lex, 'parse>, &'lex [Token<'lex>])),
    Failed((ParseError<'lex>, &'lex [Token<'lex>])),
}
type ParseCache<'lex, 'parse> = HashMap<(usize, CacheKey), CacheState<'lex, 'parse>>;

#[derive(Debug)]
pub enum Ast<'lex: 'parse, 'parse> {
    Identifier(&'lex Token<'lex>),
    FunctionCall {
        target: &'parse Ast<'lex, 'parse>,
        args: Vec<&'parse Ast<'lex, 'parse>>,
    },
}

pub fn parse_top<'lex, 'parse>(
    tokens: &'lex [Token<'lex>],
    arena: Arena<'lex, 'parse>,
) -> Result<'lex, 'parse> {
    let mut cache = HashMap::new();
    parse_expression(tokens, arena, &mut cache)
}
