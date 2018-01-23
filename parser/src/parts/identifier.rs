use ::*;

pub fn parse_identifier<'lex, 'parse>(
    tokens: &'lex [Token<'lex>],
    arena: Arena<'lex, 'parse>,
    _cache: &mut ParseCache<'lex, 'parse>,
) -> Result<'lex, 'parse> {
    let (ident, tokens) = expect_token_type!(tokens, TokenKind::Identifier(_), "identifier")?;
    Ok((arena.alloc(Ast::Identifier(ident)), tokens))
}

#[test]
fn identifier() {
    use test_util::with_parsed;

    with_parsed("abc", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::Identifier(&Token{kind: TokenKind::Identifier(ident), ..}),

            ident == "abc"
        };
    });
}
