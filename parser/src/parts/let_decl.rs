use *;

pub fn parse_arg_list<'parse>(
    mut tokens_u: &'parse [Token<'parse>],
    arena: Arena<'parse>,
) -> std::result::Result<
    (Vec<(&'parse str, AstPtr<'parse>)>, &'parse [Token<'parse>]),
    (ParseError<'parse>, &'parse [Token<'parse>]),
> {
    let mut params = vec![];
    loop {
        let (param, tokens) = parse_identifier(tokens_u, arena)?;
        if let &Ast::Identifier(_, s) = param {
            params.push((s, param));
        } else {
            unreachable!();
        }

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
    return Ok((params, tokens_u));
}

fn parse_function_params<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
    name: &'parse str,
    name_ast: AstPtr<'parse>,
) -> Result<'parse> {
    let (params, tokens) = if let Ok((_, tokens)) =
        expect_token_type!(tokens, TokenKind::CloseParen, "close parenthesis")
    {
        (vec![], tokens)
    } else {
        parse_arg_list(tokens, arena)?
    };

    let (_, tokens) = expect_token_type!(tokens, TokenKind::Equal, "= (equal)")?;
    let (body, tokens) = parse_expression(tokens, arena)?;
    let (_, tokens) = expect_token_type!(tokens, TokenKind::Semicolon, "; (semicolon)")?;
    Ok((
        arena.alloc(Ast::FunctionDecl {
            name,
            name_ast,
            params,
            body,
        }),
        tokens,
    ))
}

pub fn parse_let_decl<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
) -> Result<'parse> {
    let (_, tokens) = expect_token_type!(tokens, TokenKind::Let, "let (keyword)")?;
    let (name, tokens) = parse_identifier(tokens, arena)?;
    let name_s = if let &Ast::Identifier(_, s) = name {
        s
    } else {
        unreachable!()
    };
    let (tok, tokens) = expect_token_type!(
        tokens,
        TokenKind::OpenParen | TokenKind::Equal,
        "'(' (open paren), '=' (equals)"
    )?;
    if tok.kind == TokenKind::OpenParen {
        parse_function_params(tokens, arena, name_s, name)
    } else {
        let (expression, tokens) = parse_expression(tokens, arena)?;
        let (_, tokens) = expect_token_type!(tokens, TokenKind::Semicolon, "';' (semicolon)")?;
        Ok((
            arena.alloc(Ast::VariableDecl {
                name: name_s,
                name_ast: name,
                expression,
            }),
            tokens,
        ))
    }
}

#[test]
fn no_arg_function_decl() {
    use test_util::with_parsed_statement;

    with_parsed_statement("let abc() = 5;", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FunctionDecl{
                name: "abc",
                ref params,
                body: &Ast::Integer(_, 5),
                ..
            },
            params.len() == 0
        };
    });
}

#[test]
fn more_complicated_fn_body() {
    use test_util::with_parsed_statement;

    with_parsed_statement("let abc() = 5 + 10;", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FunctionDecl{
                name: "abc",
                ref params,
                body: &Ast::Add(_, _),
                ..
            },
            params.len() == 0
        };
    });
}

#[test]
fn fn_body_with_single_param() {
    use test_util::with_parsed_statement;

    with_parsed_statement("let abc(a) = 10;", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FunctionDecl {
                name: "abc",
                ref params,
                body: &Ast::Integer(_, 10),
                ..
            },
            params.len() == 1,
            matches!(params[0].0, "a")
        };
    });
}

#[test]
fn fn_body_with_multiple_params() {
    use test_util::with_parsed_statement;

    with_parsed_statement("let abc(a, b) = a + b;", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FunctionDecl {
                name: "abc",
                ref params,
                body: &Ast::Add(_, _),
                ..
            },
            params.len() == 2,
            matches!(params[0].0, "a"),
            matches!(params[1].0, "b")
        };
    });
}
