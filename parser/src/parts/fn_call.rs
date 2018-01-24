use ::*;

pub fn parse_function_call<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
    cache: &mut ParseCache<'parse>,
    lower: Parser,
) -> Result<'parse> {
    let (target, tokens) = lower(tokens, arena, cache)?;
    let rest: Result = do catch {
        let mut args = vec![];
        let (_, mut tokens_u) = expect_token_type!(tokens, TokenKind::OpenParen, "open parenthesis")?;

        if let Ok((_, tokens)) = expect_token_type!(tokens_u, TokenKind::CloseParen, "close parenthesis") {
            tokens_u = tokens;
        } else {
            loop {
                let (expr, tokens) = parse_expression(tokens_u, arena, cache)?;
                args.push(expr);
                let (comma_or_end, tokens) = expect_token_type!(tokens,  TokenKind::CloseParen | TokenKind::Comma, "comma or close parenthesis")?;
                tokens_u = tokens;
                if let &Token{kind: TokenKind::CloseParen, .. } = comma_or_end {
                    break;
                }
            }
        }

        Ok((
            arena.alloc(Ast::FunctionCall {
                target: target,
                args: args,
            }),
            tokens_u,
        ))
    };
    rest.or(Ok((target, tokens)))
}

#[test]
fn basic_function_call() {
    use test_util::with_parsed;

    with_parsed("abc()", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FunctionCall{ target: &Ast::Identifier(&Token{kind: TokenKind::Identifier(ident), ..}), ref args},

            args.len() == 0,
            ident == "abc"
        };
    });
}

#[test]
fn one_arg_function_call() {
    use test_util::with_parsed;

    with_parsed("abc(123)", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FunctionCall{ target: &Ast::Identifier(&Token{kind: TokenKind::Identifier(_), ..}), ref args},

            args.len() == 1,
            matches!(args[0], &Ast::Number(_))
        };
    });
}

#[test]
fn multi_arg_function_call() {
    use test_util::with_parsed;

    with_parsed("abc(123,cde)", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FunctionCall{ target: &Ast::Identifier(&Token{kind: TokenKind::Identifier(_), ..}), ref args},

            args.len() == 2,
            matches!(args[0], &Ast::Number(_)),
            matches!(args[1], &Ast::Identifier(_))
        };
    });
}

#[test]
fn nested_function_call() {
    use test_util::with_parsed;

    with_parsed("abc(def(),ghi(123))", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FunctionCall{ target: &Ast::Identifier(&Token{kind: TokenKind::Identifier(_), ..}), ref args},

            args.len() == 2,
            matches!{args[0],
                &Ast::FunctionCall{ref args, ..},
                args.len() == 0
            },
            matches!{args[1],
                &Ast::FunctionCall{ref args, ..},
                args.len() == 1
            }
        };
    });
}
