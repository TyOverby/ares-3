use *;

fn parse_pipeline_right<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
    lower: Parser,
    prev: &'parse Ast<'parse>,
) -> Result<'parse> {
    let tokens = match expect_token_type!(tokens, TokenKind::Pipeline, "|> (pipeline)") {
        Ok((_, tokens)) => tokens,
        Err(_) => return Ok((prev, tokens)),
    };
    let (right, tokens) = lower(tokens, arena)?;
    parse_pipeline_right(
        tokens,
        arena,
        lower,
        arena.alloc(Ast::Pipeline(prev, right)),
    )
}

pub fn parse_pipeline<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
    lower: Parser,
) -> Result<'parse> {
    let (left, tokens) = lower(tokens, arena)?;
    parse_pipeline_right(tokens, arena, lower, left)
}

#[test]
fn basic_pipeline() {
    use test_util::with_parsed_expression;

    with_parsed_expression("a |> c", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::Pipeline(
                &Ast::Identifier(_, "a"),
                &Ast::Identifier(_, "c"),
            )
        };
    });
}

#[test]
fn chained_pipeline() {
    use test_util::with_parsed_expression;

    with_parsed_expression("a |> b |> c", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::Pipeline(
                &Ast::Pipeline(
                    &Ast::Identifier(_, "a"),
                    &Ast::Identifier(_, "b"),
                ),
                &Ast::Identifier(_, "c"),
            )
        };
    });
}

#[test]
fn pipeline_with_anon_func() {
    use test_util::with_parsed_expression;

    with_parsed_expression("a |> (x) => x", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::Pipeline(
                &Ast::Identifier(_, "a"),
                &Ast::AnonFunc{..}
            )
        };
    });
}
