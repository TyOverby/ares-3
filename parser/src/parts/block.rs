use *;

pub fn parse_block_expression<'a>(
    tokens: &'a [Token<'a>],
    alloc: &mut Allocator<'a>,
) -> Result<'a> {
    let (_, mut tokens) = expect_token_type!(tokens, TokenKind::OpenBrace, "'{' open brace")?;

    let mut statements = vec![];
    loop {
        if let Ok((statement, tokens_n)) = parse_statement(tokens, alloc) {
            tokens = tokens_n;
            statements.push(statement);
        } else {
            break;
        }
    }

    let (expr, tokens) = parse_expression(tokens, alloc)?;
    let (_, tokens) = expect_token_type!(tokens, TokenKind::CloseBrace, "'}' close brace")?;
    let statements = alloc.alloc_iter(statements);
    return Ok((
        alloc.alloc(Ast::BlockExpr {
            statements,
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
                statements: &[&Ast::FunctionCall{..}],
                final_expression: &Ast::Identifier(_, _)
            }
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
                statements: &[
                    &Ast::FunctionCall{..},
                    &Ast::FunctionCall{..},
                ],
                final_expression: &Ast::Identifier(_, _)
            },
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
