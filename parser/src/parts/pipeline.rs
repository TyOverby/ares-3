use *;

fn parse_pipeline_right<'a>(
    tokens: &'a [Token<'a>],
    arena: &mut Allocator<'a>,
    lower: &impl Fn(&'a [Token<'a>], &mut Allocator<'a>) -> Result<'a>,
    prev: AstPtr<'a>,
) -> Result<'a> {
    let tokens = match expect_token_type!(tokens, TokenKind::Pipeline, "|> (pipeline)") {
        Ok((_, tokens)) => tokens,
        Err(_) => return Ok((prev, tokens)),
    };
    let (right, tokens) = lower(tokens, arena)?;
    let prev = arena.alloc(Ast::Pipeline(prev, right));
    parse_pipeline_right(tokens, arena, lower, prev)
}

pub fn parse_pipeline<'a>(
    tokens: &'a [Token<'a>],
    alloc: &mut Allocator<'a>,
    lower: &impl Fn(&'a [Token<'a>], &mut Allocator<'a>) -> Result<'a>,
) -> Result<'a> {
    let (left, tokens) = lower(tokens, alloc)?;
    parse_pipeline_right(tokens, alloc, lower, left)
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
