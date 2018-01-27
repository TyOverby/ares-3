use ::*;
use lexer::TokenKind;

pub fn parse_additive<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
    cache: &mut ParseCache<'parse>,
    lower: Parser,
) -> Result<'parse> {
    let (left, tokens) = lower(tokens, arena, cache)?;
    let rest: Result<'parse> = do catch {
        let (op, tokens) = expect_token_type!(
            tokens,
            TokenKind::Plus | TokenKind::Minus,
            "+ or - (add or subtract)"
        )?;
        let (right, tokens) = me_or_fallback!(parse_additive, lower, (tokens, arena, cache))?;
        if op.kind == TokenKind::Plus {
            Ok((arena.alloc(Ast::Add(left, right)), tokens))
        } else {
            Ok((arena.alloc(Ast::Sub(left, right)), tokens))
        }
    };
    rest.or(Ok((left, tokens)))
}

pub fn parse_multiplicative<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
    cache: &mut ParseCache<'parse>,
    lower: Parser,
) -> Result<'parse> {
    let (left, tokens) = lower(tokens, arena, cache)?;
    let rest: Result<'parse> = do catch {
        let (op, tokens) = expect_token_type!(
            tokens,
            TokenKind::Mul | TokenKind::Div,
            "* or / (multiply or divide)"
        )?;
        let (right, tokens) = me_or_fallback!(parse_multiplicative, lower, (tokens, arena, cache))?;
        if op.kind == TokenKind::Mul {
            Ok((arena.alloc(Ast::Mul(left, right)), tokens))
        } else {
            Ok((arena.alloc(Ast::Div(left, right)), tokens))
        }
    };
    rest.or(Ok((left, tokens)))
}

#[test]
fn test_parse_add() {
    use test_util::with_parsed_expression;

    with_parsed_expression("a+b", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::Add(&Ast::Identifier(&Token{kind: TokenKind::Identifier(left), ..}),
                      &Ast::Identifier(&Token{kind: TokenKind::Identifier(right), ..})),

            left == "a",
            right == "b"
        };
    });
}

#[test]
fn test_parse_sub() {
    use test_util::with_parsed_expression;

    with_parsed_expression("a-b", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::Sub(&Ast::Identifier(&Token{kind: TokenKind::Identifier(left), ..}),
                      &Ast::Identifier(&Token{kind: TokenKind::Identifier(right), ..})),

            left == "a",
            right == "b"
        };
    });
}

#[test]
fn test_parse_mul() {
    use test_util::with_parsed_expression;

    with_parsed_expression("a*b", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::Mul(&Ast::Identifier(&Token{kind: TokenKind::Identifier(left), ..}),
                      &Ast::Identifier(&Token{kind: TokenKind::Identifier(right), ..})),

            left == "a",
            right == "b"
        };
    });
}

#[test]
fn test_parse_div() {
    use test_util::with_parsed_expression;

    with_parsed_expression("a/b", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::Div(&Ast::Identifier(&Token{kind: TokenKind::Identifier(left), ..}),
                      &Ast::Identifier(&Token{kind: TokenKind::Identifier(right), ..})),

            left == "a",
            right == "b"
        };
    });
}

#[test]
fn order_of_operations_a() {
    use test_util::with_parsed_expression;

    with_parsed_expression("c+a*b", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::Add(&Ast::Identifier(_), &Ast::Mul(_, _)),
        };
    });
}

#[test]
fn order_of_operations_b() {
    use test_util::with_parsed_expression;

    with_parsed_expression("a*b+c", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::Add(&Ast::Mul(_, _), &Ast::Identifier(_)),
        };
    });
}

#[test]
fn chained_addition() {
    use test_util::with_parsed_expression;

    with_parsed_expression("a+b+c", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::Add(
                &Ast::Identifier(_),
                &Ast::Add(_, _)),
        };
    });
}
