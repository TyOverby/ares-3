use *;

pub fn parse_parenthesized<'a>(tokens: &'a [Token<'a>], arena: &mut Allocator<'a>) -> Result<'a> {
    let (_, tokens) = expect_token_type!(tokens, TokenKind::OpenParen, "open parenthesis")?;
    let (expr, tokens) = parse_expression(tokens, arena)?;
    let (_, tokens) = expect_token_type!(tokens, TokenKind::CloseParen, "close parenthesis")?;
    return Ok((expr, tokens));
}

#[test]
fn parenthesized_identifier() {
    use test_util::with_parsed_expression;

    with_parsed_expression("(a)", |res| {
        let (res, _) = res.unwrap();
        matches!{ res,
            &Ast::Identifier(_, "a")
        };
    });
}

#[test]
fn parenthesized_math() {
    use test_util::with_parsed_expression;

    with_parsed_expression("(a+b)*c", |res| {
        let (res, _) = res.unwrap();
        matches!{ res,
            &Ast::Mul(
                &Ast::Add(
                    &Ast::Identifier(_, "a"),
                    &Ast::Identifier(_, "b"),
                ),
                &Ast::Identifier(_, "c"),
            )
        };
    });
}

#[test]
fn nested_parens() {
    use test_util::with_parsed_expression;

    with_parsed_expression("(((((a)))))", |res| {
        let (res, _) = res.unwrap();
        matches!{ res,
            &Ast::Identifier(_, "a")
        };
    });
}

#[test]
fn nested_parens_with_function_call() {
    use test_util::with_parsed_expression;

    with_parsed_expression("(((((a)))(b)))", |res| {
        let (res, _) = res.unwrap();
        matches!{ res,
            &Ast::FunctionCall{
                target: &Ast::Identifier(_, "a"),
                ref args
            },
            args.len() == 1
        };
    });
}
