use lexer::TokenKind;
use *;

pub fn parse_additive<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
    lower: Parser,
) -> Result<'parse> {
    let (left, tokens) = lower(tokens, arena)?;
    let rest: Result<'parse> = (|| {
        let (op, tokens) = expect_token_type!(
            tokens,
            TokenKind::Plus | TokenKind::Minus,
            "+ or - (add or subtract)"
        )?;
        let (right, tokens) = me_or_fallback!(parse_additive, lower, (tokens, arena))?;
        if op.kind == TokenKind::Plus {
            Ok((arena.alloc(Ast::Add(left, right)) as &_, tokens))
        } else {
            Ok((arena.alloc(Ast::Sub(left, right)) as &_, tokens))
        }
    })();
    rest.or(Ok((left, tokens)))
}

pub fn parse_multiplicative<'parse>(
    tokens: &'parse [Token<'parse>],
    arena: Arena<'parse>,
    lower: Parser,
) -> Result<'parse> {
    let (left, tokens) = lower(tokens, arena)?;
    let rest: Result<'parse> = (|| {
        let (op, tokens) = expect_token_type!(
            tokens,
            TokenKind::Mul | TokenKind::Div,
            "* or / (multiply or divide)"
        )?;
        let (right, tokens) = me_or_fallback!(parse_multiplicative, lower, (tokens, arena))?;
        if op.kind == TokenKind::Mul {
            Ok((arena.alloc(Ast::Mul(left, right)) as &_, tokens))
        } else {
            Ok((arena.alloc(Ast::Div(left, right)) as &_, tokens))
        }
    })();
    rest.or(Ok((left, tokens)))
}

#[test]
fn test_parse_add() {
    use test_util::with_parsed_expression;

    with_parsed_expression("a+b", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::Add(&Ast::Identifier(_, "a"),
                      &Ast::Identifier(_, "b"))
        };
    });
}

#[test]
fn test_parse_sub() {
    use test_util::with_parsed_expression;

    with_parsed_expression("a-b", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::Sub(&Ast::Identifier(_, "a"),
                      &Ast::Identifier(_, "b"))
        };
    });
}

#[test]
fn test_parse_mul() {
    use test_util::with_parsed_expression;

    with_parsed_expression("a*b", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::Mul(&Ast::Identifier(_, "a"),
                      &Ast::Identifier(_, "b"))
        };
    });
}

#[test]
fn test_parse_div() {
    use test_util::with_parsed_expression;

    with_parsed_expression("a/b", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::Div(&Ast::Identifier(_, "a"),
                      &Ast::Identifier(_, "b"))
        };
    });
}

#[test]
fn order_of_operations_a() {
    use test_util::with_parsed_expression;

    with_parsed_expression("c+a*b", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::Add(&Ast::Identifier(_, "c"), &Ast::Mul(_, _)),
        };
    });
}

#[test]
fn order_of_operations_b() {
    use test_util::with_parsed_expression;

    with_parsed_expression("a*b+c", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::Add(&Ast::Mul(_, _), &Ast::Identifier(_, "c")),
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
                &Ast::Identifier(_, "a"),
                &Ast::Add(_, _)),
        };
    });
}
