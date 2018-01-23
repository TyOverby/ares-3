use ::*;

pub fn parse_base<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
    cache: &mut ParseCache<'parse>,
) -> Result<'parse> {
    if let Ok(res) = parse_identifier(tokens, arena, cache) {
        return Ok(res);
    }

    if let Ok(res) = parse_number(tokens, arena, cache) {
        return Ok(res);
    }

    if let Ok(res) = parse_parenthesized(tokens, arena, cache) {
        return Ok(res);
    }

    return Err((
        ParseError::UnexpectedToken {
            found: &tokens[0],
            expected: "identifier or number",
        },
        tokens,
    ));
}

pub fn parse_expression<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
    cache: &mut ParseCache<'parse>,
) -> Result<'parse> {
    if let Ok(res) = parse_function_call(tokens, arena, cache) {
        return Ok(res);
    }

    order_ops!(
        (tokens, arena, cache),
        parse_additive,
        parse_multiplicative,
        parse_field_access,
        parse_base
    )
}
