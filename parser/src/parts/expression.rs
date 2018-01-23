use ::*;

pub fn parse_expression<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
    cache: &mut ParseCache<'parse>,
) -> Result<'parse> {
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
