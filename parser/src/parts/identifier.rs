use ::*;

pub fn parse_identifier<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
    _cache: &mut ParseCache<'parse>,
) -> Result<'parse> {
    let (ident, tokens) = expect_token_type!(tokens, TokenKind::Identifier(_), "identifier")?;
    Ok((arena.alloc(Ast::Identifier(ident)), tokens))
}

#[test]
fn identifier() {
    use test_util::with_parsed_expression;

    with_parsed_expression("abc", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::Identifier(&Token{kind: TokenKind::Identifier(ident), ..}),

            ident == "abc"
        };
    });
}
