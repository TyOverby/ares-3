extern crate lexer;
extern crate typed_arena;

#[macro_use]
mod macros;
mod parts;
mod test_util;

use lexer::{Token, TokenKind};
pub use parts::*;

type Arena<'parse> = &'parse typed_arena::Arena<Ast<'parse>>;

pub type Parser<'a> = &'a for<'parse> Fn(&'parse [Token<'parse>], Arena<'parse>) -> Result<'parse>;

pub type AstPtr<'parse> = &'parse Ast<'parse>;

type Result<'parse> = std::result::Result<
    (AstPtr<'parse>, &'parse [Token<'parse>]),
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
pub enum ArgumentSyntax<'parse> {
    Expression(AstPtr<'parse>),
    Underscore,
}

#[derive(Debug)]
pub enum Ast<'parse> {
    Identifier(&'parse Token<'parse>, &'parse str),
    Integer(&'parse Token<'parse>, i64),
    Float(&'parse Token<'parse>, f64),
    FunctionCall {
        target: AstPtr<'parse>,
        args: Vec<ArgumentSyntax<'parse>>,
    },
    DebugCall(AstPtr<'parse>),
    Pipeline(AstPtr<'parse>, AstPtr<'parse>),
    Add(AstPtr<'parse>, AstPtr<'parse>),
    Sub(AstPtr<'parse>, AstPtr<'parse>),
    Div(AstPtr<'parse>, AstPtr<'parse>),
    Mul(AstPtr<'parse>, AstPtr<'parse>),
    AnonFunc {
        params: Vec<(&'parse str, AstPtr<'parse>)>,
        body: AstPtr<'parse>,
    },
    FunctionDecl {
        name: &'parse str,
        name_ast: AstPtr<'parse>,
        params: Vec<(&'parse str, AstPtr<'parse>)>,
        body: AstPtr<'parse>,
    },
    VariableDecl {
        name: &'parse str,
        name_ast: AstPtr<'parse>,
        expression: AstPtr<'parse>,
    },
    FieldAccess {
        target: AstPtr<'parse>,
        field: AstPtr<'parse>,
        field_name: &'parse str,
    },
    Module {
        statements: Vec<AstPtr<'parse>>,
        module_id: &'parse str,
    },
    BlockExpr {
        statements: Vec<AstPtr<'parse>>,
        final_expression: AstPtr<'parse>,
    },
}
