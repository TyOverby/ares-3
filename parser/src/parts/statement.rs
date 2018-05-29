use *;

pub fn parse_expression_statement<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: &mut Allocator<'parse>,
) -> Result<'parse> {
    let (expression, tokens) = parse_expression(tokens, arena)?;
    let (_, tokens) = expect_token_type!(tokens, TokenKind::Semicolon, "; (semicolon)")?;
    Ok((expression, tokens))
}

pub fn parse_statement<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: &mut Allocator<'parse>,
) -> Result<'parse> {
    if let Ok(res) = parse_debug_call(tokens, arena) {
        return Ok(res);
    }

    if let Ok(res) = parse_expression_statement(tokens, arena) {
        return Ok(res);
    }

    if let Ok(res) = parse_let_decl(tokens, arena) {
        return Ok(res);
    }

    return Err((
        ParseError::UnexpectedToken {
            found: &tokens[0],
            expected: "statement",
        },
        tokens,
    ));
}

#[test]
fn expression_statement() {
    use test_util::with_parsed_statement;

    with_parsed_statement("abc();", |res| {
        assert!(res.is_ok());
    });
}
