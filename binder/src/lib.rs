extern crate lexer;
extern crate parser;

use parser::Ast;

trait Binder {

}

pub enum Bound<'bound> {
    Integer { ast: Ast<'bound>, value: i64 },
    Float { ast: Ast<'bound>, value: f64 },
    Identifier {
        ast: Ast<'bound>,
        ident: &'bound str,
    },
    FunctionCall {
        ast: Ast<'bound>,
        target: &'bound Bound<'bound>,
        args: Vec<&'bound Bound<'bound>>,
    },
    Pipeline {
        ast_left: &'bound Ast<'bound>,
        ast_right: &'bound Ast<'bound>,
        left: &'bound Bound<'bound>,
        right: &'bound Bound<'bound>,
    },
    Add {
        ast_left: &'bound Ast<'bound>,
        ast_right: &'bound Ast<'bound>,
        left: &'bound Bound<'bound>,
        right: &'bound Bound<'bound>,
    },
    Sub {
        ast_left: &'bound Ast<'bound>,
        ast_right: &'bound Ast<'bound>,
        left: &'bound Bound<'bound>,
        right: &'bound Bound<'bound>,
    },
    Div {
        ast_left: &'bound Ast<'bound>,
        ast_right: &'bound Ast<'bound>,
        left: &'bound Bound<'bound>,
        right: &'bound Bound<'bound>,
    },
    Mul {
        ast_left: &'bound Ast<'bound>,
        ast_right: &'bound Ast<'bound>,
        left: &'bound Bound<'bound>,
        right: &'bound Bound<'bound>,
    },
    FunctionDecl {
        name: &'bound str,
        params: Vec<&'bound str>,
        body: &'bound Bound<'bound>,
    },
    VariableDecl {
        name: &'bound str,
        expression_ast: &'bound Ast<'bound>,
        expression: &'bound Bound<'bound>,
    },
    FieldAccess {
        target_ast: &'bound Ast<'bound>,
        field_ast: &'bound Ast<'bound>,
        field_name: &'bound str,
    },
    BlockExpr {
        statements: Vec<(&'bound Ast<'bound>, &'bound Bound<'bound>)>,
        final_expression_ast: &'bound Ast<'bound>,
        final_expression: &'bound Bound<'bound>,
    },
}
