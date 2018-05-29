use *;

fn parse_function_call_right<'a>(
    tokens: &'a [Token<'a>],
    alloc: &mut Allocator<'a>,
    lower: &impl Fn(&'a [Token<'a>], &mut Allocator<'a>) -> Result<'a>,
    prev: AstPtr<'a>,
) -> Result<'a> {
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
                let (expr, tokens) = parse_expression(tokens_u, alloc)?;
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
    let args = alloc.alloc_iter(args);
    let current = alloc.alloc(Ast::FunctionCall { target: prev, args });

    parse_function_call_right(tokens_u, alloc, lower, current)
}

pub fn parse_function_call<'a>(
    tokens: &'a [Token<'a>],
    arena: &mut Allocator<'a>,
    lower: &impl Fn(&'a [Token<'a>], &mut Allocator<'a>) -> Result<'a>,
) -> Result<'a> {
    let (left, tokens) = lower(tokens, arena)?;
    parse_function_call_right(tokens, arena, lower, left)
}

#[test]
fn basic_function_call() {
    use test_util::with_parsed_expression;

    with_parsed_expression("abc()", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FunctionCall {
                target: &Ast::Identifier(_, "abc"),
                args: &[],
            },
        };
    });
}

#[test]
fn one_arg_function_call() {
    use test_util::with_parsed_expression;

    with_parsed_expression("abc(123)", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FunctionCall {
                target: &Ast::Identifier(_, "abc"),
                args: &[ArgumentSyntax::Expression(&Ast::Integer(_, 123))],
            },
        };
    });
}

#[test]
fn two_arg_function_call_with_one_underscore() {
    use test_util::with_parsed_expression;

    with_parsed_expression("abc(123, _)", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FunctionCall {
                target: &Ast::Identifier(_, "abc"),
                args: &[
                    ArgumentSyntax::Expression(&Ast::Integer(_, 123)),
                    ArgumentSyntax::Underscore,
                ],
            },
        };
    });
}

#[test]
fn one_arg_function_call_with_underscore() {
    use test_util::with_parsed_expression;

    with_parsed_expression("abc(_)", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FunctionCall {
                target: &Ast::Identifier(_, "abc"),
                args: &[ArgumentSyntax::Underscore]
            },
        };
    });
}

#[test]
fn two_arg_function_call_with_underscore() {
    use test_util::with_parsed_expression;

    with_parsed_expression("abc(_, _)", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FunctionCall {
                target: &Ast::Identifier(_, "abc"),
                args: &[
                    ArgumentSyntax::Underscore,
                    ArgumentSyntax::Underscore
                ],
            },
        };
    });
}

#[test]
fn multi_arg_function_call() {
    use test_util::with_parsed_expression;

    with_parsed_expression("abc(123,cde)", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FunctionCall {
                target: &Ast::Identifier(_, "abc"),
                args: &[
                    ArgumentSyntax::Expression(&Ast::Integer(_, 123)),
                    ArgumentSyntax::Expression(&Ast::Identifier(_, "cde")),
                ]
            },
        };
    });
}

#[test]
fn nested_function_call() {
    use test_util::with_parsed_expression;

    with_parsed_expression("abc(def(),ghi(123))", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FunctionCall {
                target: &Ast::Identifier(_, "abc"),
                args: &[
                    ArgumentSyntax::Expression(&Ast::FunctionCall{args: &[], ..}),
                    ArgumentSyntax::Expression(&Ast::FunctionCall{args: &[_], ..}),
                ]
            },
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
