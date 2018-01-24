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
    let base: Parser = &|t, a, c| parse_base(t, a, c);
    let field_access: Parser =
        &|t, a, c| parse_field_access(t, a, c, base).or_else(|_| base(t, a, c));
    let function_call: Parser =
        &|t, a, c| parse_function_call(t, a, c, field_access);
    let multiplicative: Parser =
        &|t, a, c| parse_multiplicative(t, a, c, function_call);
    let additive: Parser =
        &|t, a, c| parse_additive(t, a, c, multiplicative);

    additive(tokens, arena, cache)
}
