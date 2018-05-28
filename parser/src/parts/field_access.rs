use *;

fn parse_field_access_right<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
    prev: AstPtr<'parse>,
) -> Result<'parse> {
    let tokens = match expect_token_type!(tokens, TokenKind::Dot, "dot") {
        Ok((_, tokens)) => tokens,
        Err(_) => return Ok((prev, tokens)),
    };
    let (right, tokens) = parse_identifier(tokens, arena)?;
    let name_s = if let &Ast::Identifier(_, s) = right {
        s
    } else {
        unreachable!()
    };
    parse_field_access_right(
        tokens,
        arena,
        arena.alloc(Ast::FieldAccess {
            target: prev,
            field: right,
            field_name: name_s,
        }),
    )
}

pub fn parse_field_access<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
    lower: Parser,
) -> Result<'parse> {
    let (left, tokens) = lower(tokens, arena)?;
    parse_field_access_right(tokens, arena, left)
}

#[test]
fn basic_field_access() {
    use test_util::with_parsed_expression;

    with_parsed_expression("a.b", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FieldAccess{
                target: &Ast::Identifier(_, "a"),
                field_name:  "b",
                ..
            }
        };
    });
}

#[test]
fn nested_field_access() {
    use test_util::with_parsed_expression;

    with_parsed_expression("a.b.c", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FieldAccess{
                target: &Ast::FieldAccess{
                    target: &Ast::Identifier(_, "a"),
                    field_name:  "b",
                    ..
                },
                field_name:  "c",
                ..
            }
        };
    });
}

#[test]
fn nested_function_call() {
    use test_util::with_parsed_expression;

    with_parsed_expression("a.b()", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FunctionCall{
                target: &Ast::FieldAccess{
                    target: &Ast::Identifier(_, "a"),
                    field_name:  "b",
                    ..
                },
                ref args,
            },
            args.len() == 0
        };
    });
}

#[test]
fn nested_field_access_with_math() {
    use test_util::with_parsed_expression;

    with_parsed_expression("a.b+c.d*e.f", |res| {
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
