use *;

pub fn parse_base<'a>(tokens: &'a [Token<'a>], arena: &mut Allocator<'a>) -> Result<'a> {
    if let Ok(res) = parse_identifier(tokens, arena) {
        return Ok(res);
    }

    if let Ok(res) = parse_number(tokens, arena) {
        return Ok(res);
    }

    if let Ok(res) = parse_anon_func(tokens, arena) {
        return Ok(res);
    }

    if let Ok(res) = parse_parenthesized(tokens, arena) {
        return Ok(res);
    }

    if let Ok(res) = parse_block_expression(tokens, arena) {
        return Ok(res);
    }

    if tokens.len() == 0 {
        return Err((ParseError::EndOfFileReached, tokens));
    }
    return Err((
        ParseError::UnexpectedToken {
            found: &tokens[0],
            expected: "identifier or number",
        },
        tokens,
    ));
}

pub fn parse_expression<'a>(tokens: &'a [Token<'a>], alloc: &mut Allocator<'a>) -> Result<'a> {
    let parser = precedence!(
        parse_pipeline,
        parse_additive,
        parse_multiplicative,
        parse_function_call,
        parse_field_access,
        parse_base
    );

    parser(tokens, alloc)
}
