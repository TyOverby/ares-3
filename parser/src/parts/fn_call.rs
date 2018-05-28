use *;

fn parse_function_call_right<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
    cache: &mut ParseCache<'parse>,
    lower: Parser,
    prev: &'parse Ast<'parse>,
) -> Result<'parse> {
    let mut tokens_u = match expect_token_type!(tokens, TokenKind::OpenParen, "open paren") {
        Ok((_, tokens)) => tokens,
        Err(_) => return Ok((prev, tokens)),
    };
    let mut args = vec![];

    if let Ok((_, tokens)) =
        expect_token_type!(tokens_u, TokenKind::CloseParen, "close parenthesis")
    {
        tokens_u = tokens;
    } else {
        loop {
            let underscore = expect_token_type!(tokens_u, TokenKind::Underscore, "_");
            let tokens = if let Ok((_, tokens)) = underscore {
                args.push(ArgumentSyntax::Underscore);
                tokens
            } else {
                let (expr, tokens) = parse_expression(tokens_u, arena, cache)?;
                args.push(ArgumentSyntax::Expression(expr));
                tokens
            };

            let (comma_or_end, tokens) = expect_token_type!(
                tokens,
                TokenKind::CloseParen | TokenKind::Comma,
                "comma or close parenthesis"
            )?;
            tokens_u = tokens;
            if let &Token {
                kind: TokenKind::CloseParen,
                ..
            } = comma_or_end
            {
                break;
            }
        }
    }

    let current = arena.alloc(Ast::FunctionCall {
        target: prev,
        args: args,
    });

    parse_function_call_right(tokens_u, arena, cache, lower, current)
}

pub fn parse_function_call<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
    cache: &mut ParseCache<'parse>,
    lower: Parser,
) -> Result<'parse> {
    let (left, tokens) = lower(tokens, arena, cache)?;
    parse_function_call_right(tokens, arena, cache, lower, left)
}

#[test]
fn basic_function_call() {
    use test_util::with_parsed_expression;

    with_parsed_expression("abc()", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FunctionCall{ target: &Ast::Identifier(_, "abc"), ref args},

            args.len() == 0
        };
    });
}

#[test]
fn one_arg_function_call() {
    use test_util::with_parsed_expression;

    with_parsed_expression("abc(123)", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FunctionCall{ target: &Ast::Identifier(_, "abc"), ref args},

            args.len() == 1,
            matches!(args[0], ArgumentSyntax::Expression(&Ast::Integer(_, 123)))
        };
    });
}

#[test]
fn two_arg_function_call_with_one_underscore() {
    use test_util::with_parsed_expression;

    with_parsed_expression("abc(123, _)", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FunctionCall{ target: &Ast::Identifier(_, "abc"), ref args},

            args.len() == 2,
            matches!(args[0], ArgumentSyntax::Expression(&Ast::Integer(_, 123))),
            matches!(args[1], ArgumentSyntax::Underscore)
        };
    });
}

#[test]
fn one_arg_function_call_with_underscore() {
    use test_util::with_parsed_expression;

    with_parsed_expression("abc(_)", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FunctionCall{ target: &Ast::Identifier(_, "abc"), ref args},

            args.len() == 1,
            matches!(args[0], ArgumentSyntax::Underscore)
        };
    });
}

#[test]
fn two_arg_function_call_with_underscore() {
    use test_util::with_parsed_expression;

    with_parsed_expression("abc(_, _)", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FunctionCall{ target: &Ast::Identifier(_, "abc"), ref args},

            args.len() == 2,
            matches!(args[0], ArgumentSyntax::Underscore),
            matches!(args[1], ArgumentSyntax::Underscore)
        };
    });
}

#[test]
fn multi_arg_function_call() {
    use test_util::with_parsed_expression;

    with_parsed_expression("abc(123,cde)", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FunctionCall{ target: &Ast::Identifier(_, "abc"), ref args},

            args.len() == 2,
            matches!(args[0], ArgumentSyntax::Expression(&Ast::Integer(_, 123))),
            matches!(args[1], ArgumentSyntax::Expression(&Ast::Identifier(_, "cde")))
        };
    });
}

#[test]
fn nested_function_call() {
    use test_util::with_parsed_expression;

    with_parsed_expression("abc(def(),ghi(123))", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FunctionCall{ target: &Ast::Identifier(_, "abc"), ref args},

            args.len() == 2,
            matches!{args[0],
                ArgumentSyntax::Expression(&Ast::FunctionCall{ref args, ..}),
                args.len() == 0
            },
            matches!{args[1],
                ArgumentSyntax::Expression(&Ast::FunctionCall{ref args, ..}),
                args.len() == 1
            }
        };
    });
}

#[test]
fn double_function_call() {
    use test_util::with_parsed_expression;

    with_parsed_expression("f()()", |res| {
        let (res, _) = res.unwrap();
        matches!{res, &Ast::FunctionCall{ target: &Ast::FunctionCall{ ..}, ..} };
    });
}
