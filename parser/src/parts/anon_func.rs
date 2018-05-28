use *;

pub fn parse_anon_func<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
) -> Result<'parse> {
    let (_, tokens) = expect_token_type!(tokens, TokenKind::OpenParen, "open parenthesis")?;
    let (params, tokens) = if let Ok((_, tokens)) =
        expect_token_type!(tokens, TokenKind::CloseParen, "close parenthesis")
    {
        (vec![], tokens)
    } else {
        parse_arg_list(tokens, arena)?
    };

    let (_, tokens) = expect_token_type!(tokens, TokenKind::WideArrow, "=>")?;
    let (body, tokens) = parse_expression(tokens, arena)?;

    Ok((arena.alloc(Ast::AnonFunc { params, body }) as &_, tokens))
}

#[test]
fn no_arg_anon_func() {
    use test_util::with_parsed_expression;

    with_parsed_expression("() => 5", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::AnonFunc{
                ref params,
                body: &Ast::Integer(_, 5),
                ..
            },
            params.len() == 0
        };
    });
}

#[test]
fn one_arg_anon_func() {
    use test_util::with_parsed_expression;

    with_parsed_expression("(a) => 5", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::AnonFunc{
                ref params,
                body: &Ast::Integer(_, 5),
                ..
            },
            params.len() == 1,
            matches!(params[0].0, "a")
        };
    });
}

#[test]
fn two_arg_anon_func() {
    use test_util::with_parsed_expression;

    with_parsed_expression("(a, b) => 5", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::AnonFunc{
                ref params,
                body: &Ast::Integer(_, 5),
                ..
            },
            params.len() == 2,
            matches!(params[0].0, "a"),
            matches!(params[1].0, "b")
        };
    });
}

#[test]
fn nested_anon_func() {
    use test_util::with_parsed_expression;

    with_parsed_expression("(a) => (b) => 5", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::AnonFunc{
                params: ref pa,
                body: &Ast::AnonFunc {
                    params: ref pb,
                    body: &Ast::Integer(_, 5),
                },
                ..
            },
            pa.len() == 1,
            pb.len() == 1,
            matches!(pa[0].0, "a"),
            matches!(pb[0].0, "b")
        };
    });
}
