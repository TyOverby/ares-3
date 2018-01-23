extern crate lexer;
extern crate typed_arena;

#[macro_use]
mod util;
mod test_util;
mod parts;

use std::collections::HashMap;
use lexer::{Token, TokenKind};
use parts::*;

type Arena<'parse> = &'parse typed_arena::Arena<Ast<'parse>>;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum CacheKey {
    Function,
}

type Result<'parse> = std::result::Result<
    (&'parse Ast<'parse>, &'parse [Token<'parse>]),
    (ParseError<'parse>, &'parse [Token<'parse>]),
>;

#[derive(Clone, Debug)]
pub enum ParseError<'parse> {
    UnexpectedToken {
        found: &'parse Token<'parse>,
        expected: &'static str,
    },
    Working,
    EndOfFileReached,
}

pub enum CacheState<'parse> {
    Working,
    Done((&'parse Ast<'parse>, &'parse [Token<'parse>])),
    Failed((ParseError<'parse>, &'parse [Token<'parse>])),
}
type ParseCache<'parse> = HashMap<(usize, CacheKey), CacheState<'parse>>;

#[derive(Debug)]
pub enum Ast<'parse> {
    Identifier(&'parse Token<'parse>),
    FunctionCall {
        target: &'parse Ast<'parse>,
        args: Vec<&'parse Ast<'parse>>,
    },
}

pub fn parse_top<'parse>(tokens: &'parse [Token<'parse>], arena: Arena<'parse>) -> Result<'parse> {
    let mut cache = HashMap::new();
    parse_expression(tokens, arena, &mut cache)
}
