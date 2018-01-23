use ::*;

pub fn parse_expression<'lex, 'parse>(
    tokens: &'lex [Token<'lex>],
    arena: Arena<'lex, 'parse>,
    cache: &mut ParseCache<'lex, 'parse>,
) -> Result<'lex, 'parse> {
    if let Ok(res) = parse_function(tokens, arena, cache) {
        return Ok(res);
    }

    if let Ok(res) = parse_identifier(tokens, arena, cache) {
        return Ok(res);
    }

    return Err((
        ParseError::UnexpectedToken {
            found: &tokens[0],
            expected: "expression",
        },
        tokens,
    ));
}
