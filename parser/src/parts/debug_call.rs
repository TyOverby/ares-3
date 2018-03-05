use ::*;

pub fn parse_debug_call<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
    cache: &mut ParseCache<'parse>,
) -> Result<'parse> {
    let (_, tokens) = expect_token_type!(tokens, TokenKind::DebugKeyword, "debug keyword")?;
    let (_, tokens) = expect_token_type!(tokens, TokenKind::OpenParen, "open parenthesis")?;
    let (value, tokens) = parse_expression(tokens, arena, cache)?;
    let (_, tokens) = expect_token_type!(tokens, TokenKind::CloseParen, "close parenthesis")?;
    let (_, tokens) = expect_token_type!(tokens, TokenKind::Semicolon, "semicolon")?;
    Ok((arena.alloc(Ast::DebugCall(value)), tokens))
}

#[test]
fn basic_debug_call_statement() {
    use test_util::with_parsed_statement;

    with_parsed_statement("debug(10);", |res| {
        let (res, _) = res.unwrap();
        matches!{res, &Ast::DebugCall(&Ast::Integer(_, 10))};
    });
}
