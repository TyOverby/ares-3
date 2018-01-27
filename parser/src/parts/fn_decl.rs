use ::*;
pub fn parse_fn_decl<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
    cache: &mut ParseCache<'parse>,
) -> Result<'parse> {
    let (_, tokens) = expect_token_type!(tokens, TokenKind::Let, "let (keyword)")?;
    let (ident, tokens) = parse_identifier(tokens, arena, cache)?;
    let (_, mut tokens_u) = expect_token_type!(tokens, TokenKind::OpenParen, "open paren")?;
    let mut params = vec![];
    if let Ok((_, tokens)) =
        expect_token_type!(tokens_u, TokenKind::CloseParen, "close parenthesis")
    {
        tokens_u = tokens;
    } else {
        loop {
            let (param, tokens) = parse_identifier(tokens_u, arena, cache)?;
            params.push(param);
            let (comma_or_end, tokens) = expect_token_type!(
                tokens,
                TokenKind::CloseParen | TokenKind::Comma,
                "comma or close parenthesis"
            )?;
            tokens_u = tokens;
            if let &Token {
                kind: TokenKind::CloseParen,
                ..
            } = comma_or_end
            {
                break;
            }
        }
    }
    let tokens = tokens_u;
    let (_, tokens) = expect_token_type!(tokens, TokenKind::Equal, "= (equal)")?;
    let (body, tokens) = parse_expression(tokens, arena, cache)?;
    let (_, tokens) = expect_token_type!(tokens, TokenKind::Semicolon, "; (semicolon)")?;

    Ok((
        arena.alloc(Ast::FunctionDecl {
            name: ident,
            params: params,
            body: body,
        }),
        tokens,
    ))
}

#[test]
fn no_arg_function_decl() {
    use test_util::with_parsed_statement;

    with_parsed_statement("let abc() = 5;", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FunctionDecl{
                name: &Ast::Identifier(&Token{kind: TokenKind::Identifier(ident), ..}),
                ref params,
                body: &Ast::Number(&Token{kind: TokenKind::Integer(5), ..})
            },
            params.len() == 0,
            ident == "abc"
        };
    });
}

#[test]
fn more_complicated_fn_body() {
    use test_util::with_parsed_statement;

    with_parsed_statement("let abc() = 5 + 10;", |res| {
        res.unwrap();
    });
}
