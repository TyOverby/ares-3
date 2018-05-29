use *;

pub fn parse_anon_func<'a>(tokens: &'a [Token<'a>], alloc: &mut Allocator<'a>) -> Result<'a> {
    let (_, tokens) = expect_token_type!(tokens, TokenKind::OpenParen, "open parenthesis")?;
    let (params, tokens) = if let Ok((_, tokens)) =
        expect_token_type!(tokens, TokenKind::CloseParen, "close parenthesis")
    {
        (&[] as &[_], tokens)
    } else {
        parse_arg_list(tokens, alloc)?
    };

    let (_, tokens) = expect_token_type!(tokens, TokenKind::WideArrow, "=>")?;
    let (body, tokens) = parse_expression(tokens, alloc)?;

    Ok((alloc.alloc(Ast::AnonFunc { params, body }) as &_, tokens))
}

#[test]
fn no_arg_anon_func() {
    use test_util::with_parsed_expression;

    with_parsed_expression("() => 5", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::AnonFunc{
                params: &[],
                body: &Ast::Integer(_, 5),
                ..
            },
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
                params: &[("a", _)],
                body: &Ast::Integer(_, 5),
                ..
            },
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
                params: &[("a", _), ("b", _)],
                body: &Ast::Integer(_, 5),
                ..
            },
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
                params: &[("a", _)],
                body: &Ast::AnonFunc {
                    params: &[("b", _)],
                    body: &Ast::Integer(_, 5),
                },
                ..
            }
        };
    });
}
