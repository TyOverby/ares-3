extern crate copy_arena;
extern crate regex;

use copy_arena::{Allocator, Arena};
use regex::Regex;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub start_byte: usize,
    pub end_byte: usize,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TokenKind<'a> {
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
    Whitespace(&'a str),
    Identifier(&'a str),
    Integer(i64),
    Float(f64),
    Error(&'a str),
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

type Table<'i, 'a> = Vec<(Regex, Box<Fn(&'i str, &mut Allocator<'a>) -> TokenKind<'a>>)>;

pub fn lex<'i, 'a>(input: &'i str, mut alloc: Allocator<'a>) -> Vec<Token<'a>> {
    let table: Vec<(
        &'static str,
        Box<Fn(&'i str, &mut Allocator<'a>) -> TokenKind<'a>>,
    )> = vec![
        (r"\(", Box::new(|_, _| TokenKind::OpenParen)),
        (r"\)", Box::new(|_, _| TokenKind::CloseParen)),
        (r"\[", Box::new(|_, _| TokenKind::OpenBracket)),
        (r"\]", Box::new(|_, _| TokenKind::CloseBracket)),
        (r"\{", Box::new(|_, _| TokenKind::OpenBrace)),
        (r"\}", Box::new(|_, _| TokenKind::CloseBrace)),
        (r";", Box::new(|_, _| TokenKind::Semicolon)),
        (r",", Box::new(|_, _| TokenKind::Comma)),
        (r"\.", Box::new(|_, _| TokenKind::Dot)),
        (r"\|>", Box::new(|_, _| TokenKind::Pipeline)),
        (r"\+", Box::new(|_, _| TokenKind::Plus)),
        (r"-", Box::new(|_, _| TokenKind::Minus)),
        (r"/", Box::new(|_, _| TokenKind::Div)),
        (
            r"(debug)($|[ \n\t\(])",
            Box::new(|_, _| TokenKind::DebugKeyword),
        ),
        (r"(let)($|[ \n\t])", Box::new(|_, _| TokenKind::Let)),
        (r"=>", Box::new(|_, _| TokenKind::WideArrow)),
        (r"=", Box::new(|_, _| TokenKind::Equal)),
        (r"\*", Box::new(|_, _| TokenKind::Mul)),
        (
            r"[ \n\t]+",
            Box::new(|s, a| TokenKind::Whitespace(a.alloc_str(s))),
        ),
        (
            r"(_)($|[^a-zA-Z0-9_])",
            Box::new(|_, _| TokenKind::Underscore),
        ),
        (
            r"[a-zA-Z_][a-zA-Z0-9_]*",
            Box::new(|s, a| TokenKind::Identifier(a.alloc_str(s))),
        ),
        (
            r"[0-9]*\.[0-9]+",
            Box::new(|s, _| TokenKind::Float(s.parse().unwrap())),
        ),
        (
            r"[0-9]+",
            Box::new(|s, _| TokenKind::Integer(s.parse().unwrap())),
        ),
        (r".", Box::new(|s, a| TokenKind::Error(a.alloc_str(s)))),
    ];

    let processed: Table<'i, 'a> = table
        .into_iter()
        .map(|(r, a)| (Regex::new(&format!("^{}", r)).unwrap(), a))
        .collect();

    fn lex_one<'i, 'a>(
        input: &'i str,
        offset: usize,
        table: &Table<'i, 'a>,
        alloc: &mut Allocator<'a>,
    ) -> Option<Token<'a>> {
        for &(ref regex, ref apply) in table {
            if let Some(captures) = regex.captures(input) {
                let capture = if captures.len() == 1 {
                    captures.get(0)
                } else {
                    captures.get(1)
                }.unwrap();

                let kind = apply(capture.as_str(), alloc);
                debug_assert_eq!(capture.start(), 0);
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
        if let Some(token) = lex_one(&input[offset..], offset, &processed, &mut alloc) {
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
    let mut arena = Arena::new();
    {
        let alloc = arena.allocator();
        assert_eq!(
            lex("(", alloc),
            vec![Token {
                kind: TokenKind::OpenParen,
                start_byte: 0,
                end_byte: 1,
            }]
        );
    }

    {
        let alloc = arena.allocator();
        assert_eq!(
            lex(")", alloc),
            vec![Token {
                kind: TokenKind::CloseParen,
                start_byte: 0,
                end_byte: 1,
            }]
        );
    }
}

#[test]
fn lex_bracket() {
    let mut arena = Arena::new();
    {
        let alloc = arena.allocator();
        assert_eq!(
            lex("[", alloc),
            vec![Token {
                kind: TokenKind::OpenBracket,
                start_byte: 0,
                end_byte: 1,
            }]
        );
    }

    {
        let alloc = arena.allocator();
        assert_eq!(
            lex("]", alloc),
            vec![Token {
                kind: TokenKind::CloseBracket,
                start_byte: 0,
                end_byte: 1,
            }]
        );
    }
}

#[test]
fn lex_brace() {
    let mut arena = Arena::new();
    {
        let alloc = arena.allocator();
        assert_eq!(
            lex("{", alloc),
            vec![Token {
                kind: TokenKind::OpenBrace,
                start_byte: 0,
                end_byte: 1,
            }]
        );
    }
    {
        let alloc = arena.allocator();
        assert_eq!(
            lex("}", alloc),
            vec![Token {
                kind: TokenKind::CloseBrace,
                start_byte: 0,
                end_byte: 1,
            }]
        );
    }
}

#[test]
fn lex_integers() {
    let mut arena = Arena::new();
    let alloc = arena.allocator();
    assert_eq!(
        lex("123", alloc),
        vec![Token {
            kind: TokenKind::Integer(123),
            start_byte: 0,
            end_byte: 3,
        }]
    );
}

#[test]
fn lex_float() {
    let mut arena = Arena::new();
    let alloc = arena.allocator();
    assert_eq!(
        lex("1.2", alloc),
        vec![Token {
            kind: TokenKind::Float(1.2),
            start_byte: 0,
            end_byte: 3,
        }]
    );
}

#[test]
fn lex_whitespace() {
    let mut arena = Arena::new();
    let alloc = arena.allocator();
    assert_eq!(
        lex(" \n\t", alloc),
        vec![Token {
            kind: TokenKind::Whitespace(" \n\t"),
            start_byte: 0,
            end_byte: 3,
        }]
    );
}

#[test]
fn lex_error() {
    let mut arena = Arena::new();
    let alloc = arena.allocator();
    assert_eq!(
        lex("ø", alloc),
        vec![Token {
            kind: TokenKind::Error("ø"),
            start_byte: 0,
            end_byte: 2,
        }]
    );
}

#[test]
fn lex_punctuation() {
    let mut arena = Arena::new();
    {
        let alloc = arena.allocator();
        assert_eq!(
            lex(".", alloc),
            vec![Token {
                kind: TokenKind::Dot,
                start_byte: 0,
                end_byte: 1,
            }]
        );
    }
    {
        let alloc = arena.allocator();
        assert_eq!(
            lex(";", alloc),
            vec![Token {
                kind: TokenKind::Semicolon,
                start_byte: 0,
                end_byte: 1,
            }]
        );
    }
    {
        let alloc = arena.allocator();
        assert_eq!(
            lex(",", alloc),
            vec![Token {
                kind: TokenKind::Comma,
                start_byte: 0,
                end_byte: 1,
            }]
        );
    }
}

#[test]
fn lex_pipeline() {
    let mut arena = Arena::new();
    let alloc = arena.allocator();
    assert_eq!(
        lex("|>", alloc),
        vec![Token {
            kind: TokenKind::Pipeline,
            start_byte: 0,
            end_byte: 2,
        }]
    );
}

#[test]
fn lex_let_with_spaces() {
    let mut arena = Arena::new();
    let alloc = arena.allocator();
    assert_eq!(
        lex("let ", alloc),
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
    let mut arena = Arena::new();
    let alloc = arena.allocator();
    assert_eq!(
        lex("debug", alloc),
        vec![Token {
            kind: TokenKind::DebugKeyword,
            start_byte: 0,
            end_byte: 5,
        }]
    );
}

#[test]
fn lex_equal() {
    let mut arena = Arena::new();
    let alloc = arena.allocator();
    assert_eq!(
        lex("=", alloc),
        vec![Token {
            kind: TokenKind::Equal,
            start_byte: 0,
            end_byte: 1,
        }]
    );
}

#[test]
fn lex_wide_arrow() {
    let mut arena = Arena::new();
    let alloc = arena.allocator();
    assert_eq!(
        lex("=>", alloc),
        vec![Token {
            kind: TokenKind::WideArrow,
            start_byte: 0,
            end_byte: 2,
        }]
    );
}

#[test]
fn lex_debug_keyword_with_spaces() {
    let mut arena = Arena::new();
    let alloc = arena.allocator();
    assert_eq!(
        lex("debug ", alloc),
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
    let mut arena = Arena::new();
    let alloc = arena.allocator();
    assert_eq!(
        lex("let", alloc),
        vec![Token {
            kind: TokenKind::Let,
            start_byte: 0,
            end_byte: 3,
        }]
    );
}

#[test]
fn identifier_that_starts_with_let() {
    let mut arena = Arena::new();
    let alloc = arena.allocator();
    assert_eq!(
        lex("letx", alloc),
        vec![Token {
            kind: TokenKind::Identifier("letx"),
            start_byte: 0,
            end_byte: 4,
        }]
    );
}

#[test]
fn lex_identifier() {
    let mut arena = Arena::new();
    let alloc = arena.allocator();
    assert_eq!(
        lex("abc", alloc),
        vec![Token {
            kind: TokenKind::Identifier("abc"),
            start_byte: 0,
            end_byte: 3,
        }]
    );
}

#[test]
fn lex_identifier_that_starts_with_underscore() {
    let mut arena = Arena::new();
    let alloc = arena.allocator();
    assert_eq!(
        lex("_abc", alloc),
        vec![Token {
            kind: TokenKind::Identifier("_abc"),
            start_byte: 0,
            end_byte: 4,
        }]
    );
}

#[test]
fn lex_underscore() {
    let mut arena = Arena::new();
    let alloc = arena.allocator();
    assert_eq!(
        lex("_", alloc),
        vec![Token {
            kind: TokenKind::Underscore,
            start_byte: 0,
            end_byte: 1,
        }]
    );
}

#[test]
fn lex_identifier_with_numbers() {
    let mut arena = Arena::new();
    let alloc = arena.allocator();
    assert_eq!(
        lex("abc1", alloc),
        vec![Token {
            kind: TokenKind::Identifier("abc1"),
            start_byte: 0,
            end_byte: 4,
        }]
    );
}

#[test]
fn lex_math() {
    let mut arena = Arena::new();
    {
        let alloc = arena.allocator();
        assert_eq!(
            lex("+", alloc),
            vec![Token {
                kind: TokenKind::Plus,
                start_byte: 0,
                end_byte: 1,
            }]
        );
    }
    {
        let alloc = arena.allocator();
        assert_eq!(
            lex("-", alloc),
            vec![Token {
                kind: TokenKind::Minus,
                start_byte: 0,
                end_byte: 1,
            }]
        );
    }
    {
        let alloc = arena.allocator();
        assert_eq!(
            lex("/", alloc),
            vec![Token {
                kind: TokenKind::Div,
                start_byte: 0,
                end_byte: 1,
            }]
        );
    }
    {
        let alloc = arena.allocator();
        assert_eq!(
            lex("*", alloc),
            vec![Token {
                kind: TokenKind::Mul,
                start_byte: 0,
                end_byte: 1,
            }]
        );
    }
}

#[test]
fn lex_multiple() {
    let mut arena = Arena::new();
    let alloc = arena.allocator();
    assert_eq!(
        lex("([{", alloc),
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
