use ::*;

pub fn parse_field_access<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
    cache: &mut ParseCache<'parse>,
    lower: Parser,
) -> Result<'parse> {
    let (mut left, mut tokens_u) = lower(tokens, arena, cache)?;
    loop {
        if let Ok((_, tokens)) = expect_token_type!(tokens_u, TokenKind::Dot, "dot") {
            let (right, tokens) = parse_identifier(tokens, arena, cache)?;
            tokens_u = tokens;
            left = arena.alloc(Ast::FieldAccess {
                target: left,
                field: right,
            });
        } else {
            break;
        }
    }
    Ok((left, tokens_u))
}

#[test]
fn basic_field_access() {
    use test_util::with_parsed;

    with_parsed("a.b", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FieldAccess{
                target: &Ast::Identifier(&Token{kind: TokenKind::Identifier(left), ..}),
                field:  &Ast::Identifier(&Token{kind: TokenKind::Identifier(right), ..})
            },
            left == "a",
            right == "b"
        };
    });
}

#[test]
fn nested_field_access() {
    use test_util::with_parsed;

    with_parsed("a.b.c", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FieldAccess{
                target: &Ast::FieldAccess{
                    target: &Ast::Identifier(&Token{kind: TokenKind::Identifier(left), ..}),
                    field:  &Ast::Identifier(&Token{kind: TokenKind::Identifier(middle), ..})
                },
                field:  &Ast::Identifier(&Token{kind: TokenKind::Identifier(right), ..})
            },
            left == "a",
            middle == "b",
            right == "c"
        };
    });
}

#[test]
fn nested_function_call() {
    use test_util::with_parsed;

    with_parsed("a.b()", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FunctionCall{
                target: &Ast::FieldAccess{
                    target: &Ast::Identifier(&Token{kind: TokenKind::Identifier(left), ..}),
                    field:  &Ast::Identifier(&Token{kind: TokenKind::Identifier(right), ..})
                },
                ref args,
            },
            left == "a",
            right == "b"
        };
    });
}

#[test]
fn broken_parse() {
    use test_util::with_parsed;

    with_parsed("a.1", |res| {
        let (res, _) = res.unwrap_err();
        matches!{res,
            ParseError::UnexpectedToken{..}
        };
    });
}

#[test]
fn nested_field_access_with_math() {
    use test_util::with_parsed;

    with_parsed("a.b+c.d*e.f", |res| {
        let (res, _) = res.unwrap();
        matches!{ res, &Ast::Add(
            &Ast::FieldAccess{..},
                &Ast::Mul(
                    &Ast::FieldAccess{..},
                    &Ast::FieldAccess{..}
                )
            )
        };
    });
}
