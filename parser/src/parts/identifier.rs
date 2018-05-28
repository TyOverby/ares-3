use ::*;

pub fn parse_identifier<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
) -> Result<'parse> {
    let (ident, tokens) = expect_token_type!(tokens, TokenKind::Identifier(_), "identifier")?;
    let s = if let &Token {
        kind: TokenKind::Identifier(s),
        ..
    } = ident
    {
        s
    } else {
        unreachable!()
    };
    Ok((arena.alloc(Ast::Identifier(ident, s)), tokens))
}

#[test]
fn identifier() {
    use test_util::with_parsed_expression;

    with_parsed_expression("abc", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::Identifier(_, "abc")
        };
    });
}
