use ::*;

pub fn parse_number<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
    _cache: &mut ParseCache<'parse>,
) -> Result<'parse> {
    let (ident, tokens) =
        expect_token_type!(tokens, TokenKind::Integer(_) | TokenKind::Float(_), "number")?;
    Ok((arena.alloc(Ast::Number(ident)), tokens))
}
