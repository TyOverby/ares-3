use super::*;
use util::with_cache;

pub fn parse_function<'lex, 'parse>(
    tokens: &'lex [Token<'lex>],
    arena: Arena<'lex, 'parse>,
    cache: &mut ParseCache<'lex, 'parse>,
) -> Result<'lex, 'parse> {
    with_cache(cache, CacheKey::Function, tokens, |cache| {
        let (target, tokens) = parse_expression(tokens, arena, cache)?;
        let (_, tokens) = expect_token_type!(tokens, TokenKind::OpenParen, "open parenthesis")?;
        let (_, tokens) = expect_token_type!(tokens, TokenKind::CloseParen, "close parenthesis")?;
        Ok((
            arena.alloc(Ast::FunctionCall {
                target: target,
                args: vec![],
            }) as &_,
            tokens,
        ))
    })
}

#[test]
fn basic_function_call() {
    use test_util::with_parsed;

    with_parsed("abc()", |res| {
        let (res, _) = res.unwrap();
        matches!{res,
            &Ast::FunctionCall{ target: &Ast::Identifier(&Token{kind: TokenKind::Identifier(ident), ..}), ref args},

            args.len() == 0,
            ident == "abc"
        };
    });
}
