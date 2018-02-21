use ::*;

pub fn parse_block_expression<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
    cache: &mut ParseCache<'parse>,
) -> Result<'parse> {
    let (_, mut tokens) = expect_token_type!(tokens, TokenKind::OpenBrace, "'{' open brace")?;

    let mut statements = vec![];
    loop {
        if let Ok((statement, tokens_n)) = parse_statement(tokens, arena, cache) {
            tokens = tokens_n;
            statements.push(statement);
        } else {
            break;
        }
    }

    let (expr, tokens) = parse_expression(tokens, arena, cache)?;
    let (_, tokens) = expect_token_type!(tokens, TokenKind::CloseBrace, "'}' close brace")?;
    return Ok((
        arena.alloc(Ast::BlockExpr {
            statements: statements,
            final_expression: expr,
        }),
        tokens,
    ));
}

#[test]
fn block_with_single_expression() {
    use test_util::with_parsed_expression;

    with_parsed_expression("{a}", |res| {
        let (res, _) = res.unwrap();
        matches!{ res,
            &Ast::BlockExpr {
                ref statements,
                final_expression: &Ast::Identifier(_, _)
            },
            statements.len() == 0
        };
    });
}

#[test]
fn block_with_a_statement_and_then_an_expression() {
    use test_util::with_parsed_expression;

    with_parsed_expression("{foo(); a}", |res| {
        let (res, _) = res.unwrap();
        matches!{ res,
            &Ast::BlockExpr {
                ref statements,
                final_expression: &Ast::Identifier(_, _)
            },
            statements.len() == 1,
            matches!(statements[0], &Ast::FunctionCall{..})
        };
    });
}

#[test]
fn block_with_multiple_statements_and_then_an_expression() {
    use test_util::with_parsed_expression;

    with_parsed_expression("{foo(); bar(); a}", |res| {
        let (res, _) = res.unwrap();
        matches!{ res,
            &Ast::BlockExpr {
                ref statements,
                final_expression: &Ast::Identifier(_, _)
            },
            statements.len() == 2,
            matches!(statements[0], &Ast::FunctionCall{..}),
            matches!(statements[1], &Ast::FunctionCall{..})
        };
    });
}

#[test]
fn nested_blocks() {
    use test_util::with_parsed_expression;

    with_parsed_expression("{{a}}", |res| {
        let (res, _) = res.unwrap();
        matches!{ res,
            &Ast::BlockExpr {
                final_expression: &Ast::BlockExpr {
                    final_expression: &Ast::Identifier(_, _),
                    ..
                },
                ..
            }
        };
    });
}
