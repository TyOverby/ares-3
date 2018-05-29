extern crate copy_arena;
extern crate lexer;

#[macro_use]
mod macros;
mod parts;
mod test_util;

use copy_arena::Allocator;
use lexer::{Token, TokenKind};
pub use parts::*;
use std::result::Result as StdResult;

pub type AstPtr<'a> = &'a Ast<'a>;
pub type Result<'a> = StdResult<(AstPtr<'a>, &'a [Token<'a>]), (ParseError<'a>, &'a [Token<'a>])>;

#[derive(Clone, Debug)]
pub enum ParseError<'a> {
    UnexpectedToken {
        found: &'a Token<'a>,
        expected: &'static str,
    },
    Working,
    EndOfFileReached,
}

#[derive(Debug, Clone, Copy)]
pub enum ArgumentSyntax<'a> {
    Expression(AstPtr<'a>),
    Underscore,
}

#[derive(Debug, Copy, Clone)]
pub enum Ast<'a> {
    Identifier(&'a Token<'a>, &'a str),
    Integer(&'a Token<'a>, i64),
    Float(&'a Token<'a>, f64),
    FunctionCall {
        target: AstPtr<'a>,
        args: &'a [ArgumentSyntax<'a>],
    },
    DebugCall(AstPtr<'a>),
    Pipeline(AstPtr<'a>, AstPtr<'a>),
    Add(AstPtr<'a>, AstPtr<'a>),
    Sub(AstPtr<'a>, AstPtr<'a>),
    Div(AstPtr<'a>, AstPtr<'a>),
    Mul(AstPtr<'a>, AstPtr<'a>),
    AnonFunc {
        params: &'a [(&'a str, AstPtr<'a>)],
        body: AstPtr<'a>,
    },
    FunctionDecl {
        name: &'a str,
        name_ast: AstPtr<'a>,
        params: &'a [(&'a str, AstPtr<'a>)],
        body: AstPtr<'a>,
    },
    VariableDecl {
        name: &'a str,
        name_ast: AstPtr<'a>,
        expression: AstPtr<'a>,
    },
    FieldAccess {
        target: AstPtr<'a>,
        field: AstPtr<'a>,
        field_name: &'a str,
    },
    Module {
        statements: &'a [AstPtr<'a>],
        module_id: &'a str,
    },
    BlockExpr {
        statements: &'a [AstPtr<'a>],
        final_expression: AstPtr<'a>,
    },
}
