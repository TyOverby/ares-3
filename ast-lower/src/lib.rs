extern crate lexer;
extern crate parser;

use lexer::Token;
use parser::AstPtr;

#[derive(Debug)]
pub enum Identifier<'parse> {
    Ident(&'parse Token<'parse>, &'parse str),
    Phantom(u32),
}

pub type LoweredAstPtr<'parse> = &'parse LoweredAst<'parse>;

#[derive(Debug)]
pub enum LoweredAst<'parse> {
    Identifier(Identifier<'parse>),
    Integer(&'parse Token<'parse>, i64),
    Float(&'parse Token<'parse>, f64),
    FunctionCall {
        target: AstPtr<'parse>,
        args: Vec<AstPtr<'parse>>,
    },
    DebugCall(AstPtr<'parse>),
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
