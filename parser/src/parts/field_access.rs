use ::*;

fn parse_fields<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
    cache: &mut ParseCache<'parse>,
    prev: &'parse Ast<'parse>,
) -> Result<'parse> {
    let tokens = match expect_token_type!(tokens, TokenKind::Dot, "dot") {
        Ok((_, tokens)) => tokens,
        Err(_) => return Ok((prev, tokens)),
    };
    let (right, tokens) = parse_identifier(tokens, arena, cache)?;
    parse_fields(
        tokens,
        arena,
        cache,
        arena.alloc(Ast::FieldAccess {
            target: prev,
            field: right,
        }),
    )
}

pub fn parse_field_access<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
    cache: &mut ParseCache<'parse>,
    lower: Parser,
) -> Result<'parse> {
    let (left, tokens) = lower(tokens, arena, cache)?;
    parse_fields(tokens, arena, cache, left)
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
            right == "b",
            args.len() == 0
        };
    });
}

/*
fn broken_parse() {
    use test_util::with_parsed;

    with_parsed("a.1", |res| {
        let (res, _) = res.unwrap_err();
        matches!{res,
            ParseError::UnexpectedToken{..}
        };
    });
}*/

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
