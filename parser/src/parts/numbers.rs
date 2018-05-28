use ::*;

pub fn parse_number<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
) -> Result<'parse> {
    let (ident, tokens) =
        expect_token_type!(tokens, TokenKind::Integer(_) | TokenKind::Float(_), "number")?;
    match ident.kind {
        TokenKind::Integer(i) => Ok((arena.alloc(Ast::Integer(ident, i)), tokens)),
        TokenKind::Float(f) => Ok((arena.alloc(Ast::Float(ident, f)), tokens)),
        _ => unreachable!(),
    }
}

#[test]
fn parse_integer() {
    use test_util::with_parsed_expression;

    with_parsed_expression("123", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::Integer(_, 123)
        };
    });
}

#[test]
fn parse_float() {
    use test_util::with_parsed_expression;

    with_parsed_expression("1.23", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::Float(_, v),
            v == 1.23
        };
    });
}
