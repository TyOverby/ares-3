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
    let mut cache = HashMap::new();
    let parsed = parse_statement(&lexed, &parse_arena, &mut cache).unwrap();
    let bound = bind_top(&bind_arena, 0, parsed.0);
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
            Bound::Add {
                left: &Bound::Integer{value: 1, ..},
                right: &Bound::Integer{value: 2, ..},
                ..
            }
        );
    });
}

#[test]
fn bind_module_variable_decl() {
    with_bind("let x = 5;", |res| {
        let r = res.unwrap();
        matches!(r, Bound::VariableDecl {
            name: "x",
            expression: &Bound::Integer{value: 5, ..},
            location: BindingKind::Module{..},
            ..
        });
    });
}

#[test]
fn bind_module_fn_decl() {
    with_bind("let x(y) = 5;", |res| {
        let r = res.unwrap();
        matches!(r, Bound::FunctionDecl{
            name: "x",
            body: &Bound::Integer{value: 5, ..},
            location: BindingKind::Module{..},
            params,
            ..
        },
        params[0].0 == "y");
    });

    with_bind("let x(y) = y;", |res| {
        let r = res.unwrap();
        matches!(r, Bound::FunctionDecl{
            name: "x",
            body: &Bound::Identifier{binding_kind: BindingKind::Argument(0), ..},
            location: BindingKind::Module{..},
            params,
            ..
        },
        params[0].0 == "y");
    });

    with_bind("let x(y) = z;", |res| {
        matches!(res,
            Err(Error::UnboundIdentifier(ref s)),
            &*s == "z"
        );
    });
}