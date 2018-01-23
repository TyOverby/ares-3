use ::*;

pub fn parse_parenthesized<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
    cache: &mut ParseCache<'parse>,
) -> Result<'parse> {
    let (_, tokens) = expect_token_type!(tokens, TokenKind::OpenParen, "open parenthesis")?;
    let (expr, tokens) = parse_expression(tokens, arena, cache)?;
    let (_, tokens) = expect_token_type!(tokens, TokenKind::CloseParen, "close parenthesis")?;
    return Ok((expr, tokens));
}

#[test]
fn parenthesized_identifier() {
    use test_util::with_parsed;

    with_parsed("(a)", |res| {
        let (res, _) = res.unwrap();
        matches!{ res,
            &Ast::Identifier(&Token{kind: TokenKind::Identifier(id), ..}),
            id == "a"
        };
    });
}

#[test]
fn parenthesized_math() {
    use test_util::with_parsed;

    with_parsed("(a+b)*c", |res| {
        let (res, _) = res.unwrap();
        matches!{ res,
            &Ast::Mul(
                &Ast::Add(
                    &Ast::Identifier(_),
                    &Ast::Identifier(_),
                ),
                &Ast::Identifier(_),
            )
        };
    });
}

#[test]
fn nested_parens() {
    use test_util::with_parsed;

    with_parsed("(((((a)))))", |res| {
        let (res, _) = res.unwrap();
        matches!{ res,
            &Ast::Identifier(&Token{kind: TokenKind::Identifier(id), ..}),
            id == "a"
        };
    });
}

#[test]
fn nested_parens_with_function_call() {
    use test_util::with_parsed;

    with_parsed("(((((a)))(b)))", |res| {
        let (res, _) = res.unwrap();
        matches!{ res,
            &Ast::FunctionCall{
                target: &Ast::Identifier(&Token{kind: TokenKind::Identifier(id), ..}),
                ref args
            },
            id == "a",
            args.len() == 1

        };
    });
}
