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
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Semicolon,
    Comma,
    Dot,
    Whitespace(&'lex str),
    Identifier(&'lex str),
    Integer(i64),
    Float(f64),
    Error(&'lex str),
}

pub fn lex<'lex>(input: &'lex str) -> Vec<Token> {
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
        (r"[ \n\t]+", Box::new(|s| TokenKind::Whitespace(s))),
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
            if let Some(m) = regex.find(input) {
                let kind = apply(m.as_str());
                assert_eq!(m.start(), 0);
                return Some(Token {
                    start_byte: offset,
                    end_byte: offset + m.end(),
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
        vec![
            Token {
                kind: TokenKind::OpenParen,
                start_byte: 0,
                end_byte: 1,
            },
        ]
    );

    assert_eq!(
        lex(")"),
        vec![
            Token {
                kind: TokenKind::CloseParen,
                start_byte: 0,
                end_byte: 1,
            },
        ]
    );
}

#[test]
fn lex_bracket() {
    assert_eq!(
        lex("["),
        vec![
            Token {
                kind: TokenKind::OpenBracket,
                start_byte: 0,
                end_byte: 1,
            },
        ]
    );

    assert_eq!(
        lex("]"),
        vec![
            Token {
                kind: TokenKind::CloseBracket,
                start_byte: 0,
                end_byte: 1,
            },
        ]
    );
}

#[test]
fn lex_brace() {
    assert_eq!(
        lex("{"),
        vec![
            Token {
                kind: TokenKind::OpenBrace,
                start_byte: 0,
                end_byte: 1,
            },
        ]
    );

    assert_eq!(
        lex("}"),
        vec![
            Token {
                kind: TokenKind::CloseBrace,
                start_byte: 0,
                end_byte: 1,
            },
        ]
    );
}

#[test]
fn lex_integers() {
    assert_eq!(
        lex("123"),
        vec![
            Token {
                kind: TokenKind::Integer(123),
                start_byte: 0,
                end_byte: 3,
            },
        ]
    );
}

#[test]
fn lex_float() {
    assert_eq!(
        lex("1.2"),
        vec![
            Token {
                kind: TokenKind::Float(1.2),
                start_byte: 0,
                end_byte: 3,
            },
        ]
    );
}

#[test]
fn lex_whitespace() {
    assert_eq!(
        lex(" \n\t"),
        vec![
            Token {
                kind: TokenKind::Whitespace(" \n\t"),
                start_byte: 0,
                end_byte: 3,
            },
        ]
    );
}

#[test]
fn lex_error() {
    assert_eq!(
        lex("ø"),
        vec![
            Token {
                kind: TokenKind::Error("ø"),
                start_byte: 0,
                end_byte: 2,
            },
        ]
    );
}

#[test]
fn lex_punctuation() {
    assert_eq!(
        lex("."),
        vec![
            Token {
                kind: TokenKind::Dot,
                start_byte: 0,
                end_byte: 1,
            },
        ]
    );
    assert_eq!(
        lex(";"),
        vec![
            Token {
                kind: TokenKind::Semicolon,
                start_byte: 0,
                end_byte: 1,
            },
        ]
    );
    assert_eq!(
        lex(","),
        vec![
            Token {
                kind: TokenKind::Comma,
                start_byte: 0,
                end_byte: 1,
            },
        ]
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
