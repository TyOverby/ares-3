extern crate regex;

use regex::Regex;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Token<'lex> {
    pub kind: TokenKind<'lex>,
    pub start_byte: usize,
    pub end_byte: usize,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TokenKind<'lex> {
    DebugKeyword,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Semicolon,
    Comma,
    Pipeline,
    Dot,
    Plus,
    Minus,
    Div,
    Mul,
    Let,
    Underscore,
    WideArrow,
    Equal,
    Whitespace(&'lex str),
    Identifier(&'lex str),
    Integer(i64),
    Float(f64),
    Error(&'lex str),
}

pub fn remove_whitespace(tokens: &mut Vec<Token>) {
    tokens.retain(|token| {
        if let TokenKind::Whitespace(_) = token.kind {
            false
        } else {
            true
        }
    })
}

pub fn lex<'lex>(input: &'lex str) -> Vec<Token<'lex>> {
    let table: Vec<(&'static str, Box<Fn(&'lex str) -> TokenKind<'lex>>)> = vec![
        (r"\(", Box::new(|_| TokenKind::OpenParen)),
        (r"\)", Box::new(|_| TokenKind::CloseParen)),
        (r"\[", Box::new(|_| TokenKind::OpenBracket)),
        (r"\]", Box::new(|_| TokenKind::CloseBracket)),
        (r"\{", Box::new(|_| TokenKind::OpenBrace)),
        (r"\}", Box::new(|_| TokenKind::CloseBrace)),
        (r";", Box::new(|_| TokenKind::Semicolon)),
        (r",", Box::new(|_| TokenKind::Comma)),
        (r"\.", Box::new(|_| TokenKind::Dot)),
        (r"\|>", Box::new(|_| TokenKind::Pipeline)),
        (r"\+", Box::new(|_| TokenKind::Plus)),
        (r"-", Box::new(|_| TokenKind::Minus)),
        (r"/", Box::new(|_| TokenKind::Div)),
        (
            r"(debug)($|[ \n\t\(])",
            Box::new(|_| TokenKind::DebugKeyword),
        ),
        (r"(let)($|[ \n\t])", Box::new(|_| TokenKind::Let)),
        (r"=>", Box::new(|_| TokenKind::WideArrow)),
        (r"=", Box::new(|_| TokenKind::Equal)),
        (r"\*", Box::new(|_| TokenKind::Mul)),
        (r"[ \n\t]+", Box::new(|s| TokenKind::Whitespace(s))),
        (r"(_)($|[^a-zA-Z0-9_])", Box::new(|_| TokenKind::Underscore)),
        (r"[a-zA-Z_][a-zA-Z0-9_]*", Box::new(TokenKind::Identifier)),
        (
            r"[0-9]*\.[0-9]+",
            Box::new(|s| TokenKind::Float(s.parse().unwrap())),
        ),
        (
            r"[0-9]+",
            Box::new(|s| TokenKind::Integer(s.parse().unwrap())),
        ),
        (r".", Box::new(|s| TokenKind::Error(s))),
    ];

    let processed: Vec<_> = table
        .into_iter()
        .map(|(r, a)| (Regex::new(&format!("^{}", r)).unwrap(), a))
        .collect();

    let lex_one = |input, offset| {
        for &(ref regex, ref apply) in &processed {
            if let Some(captures) = regex.captures(input) {
                let capture = if captures.len() == 1 {
                    captures.get(0)
                } else {
                    captures.get(1)
                }.unwrap();

                let kind = apply(capture.as_str());
                assert_eq!(capture.start(), 0);
                return Some(Token {
                    start_byte: offset,
                    end_byte: offset + capture.end(),
                    kind: kind,
                });
            }
        }
        return None;
    };

    let mut offset = 0;
    let mut out = vec![];
    loop {
        if let Some(token) = lex_one(&input[offset..], offset) {
            offset = token.end_byte;
            out.push(token);
        } else {
            break;
        }
    }
    return out;
}

#[test]
fn lex_parens() {
    assert_eq!(
        lex("("),
        vec![Token {
            kind: TokenKind::OpenParen,
            start_byte: 0,
            end_byte: 1,
        }]
    );

    assert_eq!(
        lex(")"),
        vec![Token {
            kind: TokenKind::CloseParen,
            start_byte: 0,
            end_byte: 1,
        }]
    );
}

#[test]
fn lex_bracket() {
    assert_eq!(
        lex("["),
        vec![Token {
            kind: TokenKind::OpenBracket,
            start_byte: 0,
            end_byte: 1,
        }]
    );

    assert_eq!(
        lex("]"),
        vec![Token {
            kind: TokenKind::CloseBracket,
            start_byte: 0,
            end_byte: 1,
        }]
    );
}

#[test]
fn lex_brace() {
    assert_eq!(
        lex("{"),
        vec![Token {
            kind: TokenKind::OpenBrace,
            start_byte: 0,
            end_byte: 1,
        }]
    );

    assert_eq!(
        lex("}"),
        vec![Token {
            kind: TokenKind::CloseBrace,
            start_byte: 0,
            end_byte: 1,
        }]
    );
}

#[test]
fn lex_integers() {
    assert_eq!(
        lex("123"),
        vec![Token {
            kind: TokenKind::Integer(123),
            start_byte: 0,
            end_byte: 3,
        }]
    );
}

#[test]
fn lex_float() {
    assert_eq!(
        lex("1.2"),
        vec![Token {
            kind: TokenKind::Float(1.2),
            start_byte: 0,
            end_byte: 3,
        }]
    );
}

#[test]
fn lex_whitespace() {
    assert_eq!(
        lex(" \n\t"),
        vec![Token {
            kind: TokenKind::Whitespace(" \n\t"),
            start_byte: 0,
            end_byte: 3,
        }]
    );
}

#[test]
fn lex_error() {
    assert_eq!(
        lex("ø"),
        vec![Token {
            kind: TokenKind::Error("ø"),
            start_byte: 0,
            end_byte: 2,
        }]
    );
}

#[test]
fn lex_punctuation() {
    assert_eq!(
        lex("."),
        vec![Token {
            kind: TokenKind::Dot,
            start_byte: 0,
            end_byte: 1,
        }]
    );
    assert_eq!(
        lex(";"),
        vec![Token {
            kind: TokenKind::Semicolon,
            start_byte: 0,
            end_byte: 1,
        }]
    );
    assert_eq!(
        lex(","),
        vec![Token {
            kind: TokenKind::Comma,
            start_byte: 0,
            end_byte: 1,
        }]
    );
}

#[test]
fn lex_pipeline() {
    assert_eq!(
        lex("|>"),
        vec![Token {
            kind: TokenKind::Pipeline,
            start_byte: 0,
            end_byte: 2,
        }]
    );
}

#[test]
fn lex_let_with_spaces() {
    assert_eq!(
        lex("let "),
        vec![
            Token {
                kind: TokenKind::Let,
                start_byte: 0,
                end_byte: 3,
            },
            Token {
                kind: TokenKind::Whitespace(" "),
                start_byte: 3,
                end_byte: 4,
            },
        ]
    );
}

#[test]
fn lex_debug_keyword() {
    assert_eq!(
        lex("debug"),
        vec![Token {
            kind: TokenKind::DebugKeyword,
            start_byte: 0,
            end_byte: 5,
        }]
    );
}

#[test]
fn lex_equal() {
    assert_eq!(
        lex("="),
        vec![Token {
            kind: TokenKind::Equal,
            start_byte: 0,
            end_byte: 1,
        }]
    );
}

#[test]
fn lex_wide_arrow() {
    assert_eq!(
        lex("=>"),
        vec![Token {
            kind: TokenKind::WideArrow,
            start_byte: 0,
            end_byte: 2,
        }]
    );
}

#[test]
fn lex_debug_keyword_with_spaces() {
    assert_eq!(
        lex("debug "),
        vec![
            Token {
                kind: TokenKind::DebugKeyword,
                start_byte: 0,
                end_byte: 5,
            },
            Token {
                kind: TokenKind::Whitespace(" "),
                start_byte: 5,
                end_byte: 6,
            },
        ]
    );
}

#[test]
fn lex_let() {
    assert_eq!(
        lex("let"),
        vec![Token {
            kind: TokenKind::Let,
            start_byte: 0,
            end_byte: 3,
        }]
    );
}

#[test]
fn identifier_that_starts_with_let() {
    assert_eq!(
        lex("letx"),
        vec![Token {
            kind: TokenKind::Identifier("letx"),
            start_byte: 0,
            end_byte: 4,
        }]
    );
}

#[test]
fn lex_identifier() {
    assert_eq!(
        lex("abc"),
        vec![Token {
            kind: TokenKind::Identifier("abc"),
            start_byte: 0,
            end_byte: 3,
        }]
    );
}

#[test]
fn lex_identifier_that_starts_with_underscore() {
    assert_eq!(
        lex("_abc"),
        vec![Token {
            kind: TokenKind::Identifier("_abc"),
            start_byte: 0,
            end_byte: 4,
        }]
    );
}

#[test]
fn lex_underscore() {
    assert_eq!(
        lex("_"),
        vec![Token {
            kind: TokenKind::Underscore,
            start_byte: 0,
            end_byte: 1,
        }]
    );
}

#[test]
fn lex_identifier_with_numbers() {
    assert_eq!(
        lex("abc1"),
        vec![Token {
            kind: TokenKind::Identifier("abc1"),
            start_byte: 0,
            end_byte: 4,
        }]
    );
}

#[test]
fn lex_math() {
    assert_eq!(
        lex("+"),
        vec![Token {
            kind: TokenKind::Plus,
            start_byte: 0,
            end_byte: 1,
        }]
    );
    assert_eq!(
        lex("-"),
        vec![Token {
            kind: TokenKind::Minus,
            start_byte: 0,
            end_byte: 1,
        }]
    );
    assert_eq!(
        lex("/"),
        vec![Token {
            kind: TokenKind::Div,
            start_byte: 0,
            end_byte: 1,
        }]
    );
    assert_eq!(
        lex("*"),
        vec![Token {
            kind: TokenKind::Mul,
            start_byte: 0,
            end_byte: 1,
        }]
    );
}

#[test]
fn lex_multiple() {
    assert_eq!(
        lex("([{"),
        vec![
            Token {
                kind: TokenKind::OpenParen,
                start_byte: 0,
                end_byte: 1,
            },
            Token {
                kind: TokenKind::OpenBracket,
                start_byte: 1,
                end_byte: 2,
            },
            Token {
                kind: TokenKind::OpenBrace,
                start_byte: 2,
                end_byte: 3,
            },
        ]
    );
}
