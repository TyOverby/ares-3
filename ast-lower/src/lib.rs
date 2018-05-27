extern crate lexer;
extern crate parser;

use lexer::Token;
use parser::Ast;

#[derive(Debug)]
pub enum LoweredAst<'parse> {
    Identifier(&'parse Token<'parse>, &'parse str),
    Integer(&'parse Token<'parse>, i64),
    Float(&'parse Token<'parse>, f64),
    FunctionCall {
        target: &'parse Ast<'parse>,
        args: Vec<&'parse Ast<'parse>>,
    },
    DebugCall(&'parse Ast<'parse>),
    Pipeline(&'parse Ast<'parse>, &'parse Ast<'parse>),
    Add(&'parse Ast<'parse>, &'parse Ast<'parse>),
    Sub(&'parse Ast<'parse>, &'parse Ast<'parse>),
    Div(&'parse Ast<'parse>, &'parse Ast<'parse>),
    Mul(&'parse Ast<'parse>, &'parse Ast<'parse>),
    AnonFunc {
        params: Vec<(&'parse str, &'parse Ast<'parse>)>,
        body: &'parse Ast<'parse>,
    },
    FunctionDecl {
        name: &'parse str,
        name_ast: &'parse Ast<'parse>,
        params: Vec<(&'parse str, &'parse Ast<'parse>)>,
        body: &'parse Ast<'parse>,
    },
    VariableDecl {
        name: &'parse str,
        name_ast: &'parse Ast<'parse>,
        expression: &'parse Ast<'parse>,
    },
    FieldAccess {
        target: &'parse Ast<'parse>,
        field: &'parse Ast<'parse>,
        field_name: &'parse str,
    },
    Module {
        statements: Vec<&'parse Ast<'parse>>,
        module_id: &'parse str,
    },
    BlockExpr {
        statements: Vec<&'parse Ast<'parse>>,
        final_expression: &'parse Ast<'parse>,
    },
}
