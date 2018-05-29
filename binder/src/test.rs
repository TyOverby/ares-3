use super::*;

use lexer::*;
use parser::*;

macro_rules! matches {
    ($value: expr, $pattern: pat) => {
        match $value {
            $pattern => true,
            _ => panic!("match failed, actual: {:#?}", $value),
        }
    };
    ($value: expr, $pattern: pat, $($exp: expr),*) => {
        match $value {
            $pattern => {$(assert!($exp));* ; true},
            _ => panic!("match failed, actual: {:#?}", $value),
        }
    };
}

fn with_bind<F>(program: &'static str, f: F)
where
    F: for<'a> FnOnce(Result<Bound<'a>, Error>),
{
    use typed_arena::Arena;
    let mut lexed = lex(program);
    let parse_arena = Arena::new();
    let bind_arena = Arena::new();

    remove_whitespace(&mut lexed);
    let parsed = parse_module(&lexed, "my_module", &parse_arena).unwrap();
    let bound = bind_top(&bind_arena, parsed.0);
    f(bound)
}

fn remove_whitespace(tokens: &mut Vec<Token>) {
    tokens.retain(|token| {
        if let TokenKind::Whitespace(_) = token.kind {
            false
        } else {
            true
        }
    })
}

#[test]
fn bind_binary_operator() {
    with_bind("1 + 2;", |res| {
        let r = res.unwrap();
        matches!(r,
            Bound::Module { statements, .. },
            statements.len() == 1,
            matches!(statements[0],
                Bound::Add {
                    left: &Bound::Integer{value: 1, ..},
                    right: &Bound::Integer{value: 2, ..},
                    ..
                }
            )
        );
    });
}

#[test]
fn bind_module_variable_decl() {
    with_bind("let x = 5;", |res| {
        let r = res.unwrap();
        matches!(r,
            Bound::Module { statements, .. },
            statements.len() == 1,
            matches!(statements[0],
                Bound::VariableDecl {
                    name: "x",
                    expression: &Bound::Integer{value: 5, ..},
                    location: BindingKind::Module{..},
                    ..
                }
            )
        );
    });
}

#[test]
fn bind_module_fn_decl() {
    with_bind("let x(y) = 5;", |res| {
        let r = res.unwrap();
        matches!(r,
            Bound::Module{ statements, .. },
            matches!(&statements[0],
                &Bound::FunctionDecl{
                    name: "x",
                    body: &Bound::Integer{value: 5, ..},
                    location: BindingKind::Module{..},
                    ref params,
                    ..
                },
                params[0].0 == DeclarationKind::Named("y".into()))
        );
    });
}

#[test]
fn bind_module_fn_decl_with_param_reference() {
    with_bind("let x(y) = y;", |res| {
        let r = res.unwrap();
        matches!(r,
            Bound::Module{ statements, .. },
            statements.len() == 1,
            matches!(&statements[0],
                &Bound::FunctionDecl {
                    name: "x",
                    body: &Bound::Identifier{binding_kind: BindingKind::Argument(0), ..},
                    location: BindingKind::Module{..},
                    ref params,
                    ..
                },
                params[0].0 == DeclarationKind::Named("y".into()))
        );
    });
}

#[test]
fn bind_module_fn_decl_with_some_locals() {
    with_bind("let x() = { let a = 5; let b = 10; a + b };", |res| {
        let r = res.unwrap();
        matches!(r,
            Bound::Module{ statements, .. },
            statements.len() == 1
        );
    });
}

#[test]
fn bind_module_fn_decl_with_some_locals_bad() {
    with_bind("let x() = { let a = 5; let b = 10; a + c };", |res| {
        assert!(res.is_err());
    });
}

#[test]
fn bind_module_fn_decl_with_bad_reference() {
    with_bind("let x(y) = z;", |res| {
        matches!(res, Err(Error::UnboundIdentifier(ref s)), &*s == "z");
    });
}

#[test]
fn bind_module_fn_decl_with_inner_function() {
    with_bind("let f(x) = { let g() = 10; g()};", |res| {
        assert!(res.is_ok());
    });
}

#[test]
fn bind_module_fn_decl_with_upvar() {
    with_bind("let f(x) = { let g() = x; g()};", |res| {
        assert!(res.is_ok());
    });
}

#[test]
fn bind_upvar_to_module_fn() {
    with_bind("let x = 10; let f() = x;", |res| {
        assert!(res.is_ok());
    });
}
