#![feature(catch_expr)]
extern crate lexer;
extern crate typed_arena;

#[macro_use]
mod macros;
mod util;
mod test_util;
mod parts;

use std::collections::HashMap;
use lexer::{Token, TokenKind};
pub use parts::*;

type Arena<'parse> = &'parse typed_arena::Arena<Ast<'parse>>;

pub type Parser<'a> = &'a for<'parse> Fn(
    &'parse [Token<'parse>],
    Arena<'parse>,
    &mut ParseCache<'parse>,
) -> Result<'parse>;


#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum CacheKey {
    Function,
    Additive,
    Multiplicative,
    Expression,
    FieldAccess,
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

#[derive(Debug)]
pub enum CacheState<'parse> {
    Working,
    Done((&'parse Ast<'parse>, &'parse [Token<'parse>])),
    Failed((ParseError<'parse>, &'parse [Token<'parse>])),
}
type ParseCache<'parse> = HashMap<(usize, CacheKey), CacheState<'parse>>;

#[derive(Debug)]
pub enum Ast<'parse> {
    Identifier(&'parse Token<'parse>),
    Number(&'parse Token<'parse>),
    FunctionCall {
        target: &'parse Ast<'parse>,
        args: Vec<&'parse Ast<'parse>>,
    },
    Add(&'parse Ast<'parse>, &'parse Ast<'parse>),
    Pipeline(&'parse Ast<'parse>, &'parse Ast<'parse>),
    Sub(&'parse Ast<'parse>, &'parse Ast<'parse>),
    Div(&'parse Ast<'parse>, &'parse Ast<'parse>),
    Mul(&'parse Ast<'parse>, &'parse Ast<'parse>),
    FunctionDecl{
        name: &'parse Ast<'parse>,
        params: Vec<&'parse Ast<'parse>>,
        body: &'parse Ast<'parse>,
    },
    FieldAccess {
        target: &'parse Ast<'parse>,
        field: &'parse Ast<'parse>,
    },
}

