#![feature(fnbox)]

extern crate lexer;
extern crate parser;
extern crate typed_arena;
#[cfg(test)]
extern crate difference;

#[cfg(test)]
mod test;
mod binary;
mod call;

use parser::Ast;
use std::boxed::FnBox;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter, Result as FmtResult, Write};
use typed_arena::Arena;

pub type ContAstPtr<'parse> = &'parse ContAst<'parse>;
type WithContinue<'c> = Box<FnBox(Terminal<'c>) -> ContAstPtr<'c> + 'c>;

pub struct IdGet {
    id: RefCell<u32>,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Ident<'parse> {
    Identifier(&'parse str),
    Phantom(u32),
}

#[derive(PartialEq, Copy, Clone)]
pub enum Terminal<'parse> {
    Integer(i64),
    Float(f64),
    Ident(Ident<'parse>),
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum PrimOpKind {
    Term,
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(PartialEq, Debug)]
pub struct Function<'a> {
    name: Ident<'a>,
    params: Vec<Ident<'a>>,
    body: ContAstPtr<'a>,
}

#[derive(PartialEq)]
pub enum ContAst<'parse> {
    Fix {
        functions: Vec<Function<'parse>>,
        continuation: ContAstPtr<'parse>,
    },
    Call {
        target: Terminal<'parse>,
        params: Vec<Terminal<'parse>>,
        continuation: Terminal<'parse>
    },
    Primop {
        op: PrimOpKind,
        terminals: Vec<Terminal<'parse>>,
        exports: Vec<Ident<'parse>>,
        continuations: Vec<ContAstPtr<'parse>>,
    },
}

pub fn translate<'c>(
    ast: &'c Ast<'c>,
    c: WithContinue<'c>,
    idg: &'c IdGet,
    arena: &'c Arena<ContAst<'c>>,
) -> ContAstPtr<'c> {
    match ast {
        &Ast::Add(l, r) => binary::do_binary(l, r, c, PrimOpKind::Add, idg, arena),
        &Ast::Mul(l, r) => binary::do_binary(l, r, c, PrimOpKind::Mul, idg, arena),
        &Ast::Sub(l, r) => binary::do_binary(l, r, c, PrimOpKind::Sub, idg, arena),
        &Ast::Div(l, r) => binary::do_binary(l, r, c, PrimOpKind::Div, idg, arena),
        &Ast::Integer(_, i) => c(Terminal::Integer(i)),
        &Ast::Float(_, f) => c(Terminal::Float(f)),
        &Ast::Identifier(_, s) => c(Terminal::Ident(Ident::Identifier(s))),
        &Ast::FunctionCall{ref target, ref args} => call::do_call(target, &*args, c, idg, arena),
        _ => unimplemented!(),
    }
}

impl IdGet {
    pub fn new() -> IdGet {
        IdGet {
            id: RefCell::new(0),
        }
    }

    fn get<'a>(&self) -> Ident<'a> {
        let mut idm = self.id.borrow_mut();
        let r = Ident::Phantom(*idm);
        *idm += 1;
        r
    }
}

impl<'a> Debug for ContAst<'a> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        smart_print(self, f, 0)
    }
}

impl<'a> Debug for Terminal<'a> {
    fn fmt(&self, out: &mut Formatter) -> FmtResult {
        match self {
            &Terminal::Float(f) => write!(out, "{}", f),
            &Terminal::Integer(i) => write!(out, "{}", i),
            &Terminal::Ident(i) => write!(out, "{:?}", i),
        }
    }
}

impl<'a> Debug for Ident<'a> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Ident::Identifier(id) => write!(f, "{}", id),
            Ident::Phantom(n) => write!(f, "id_{}", n),
        }
    }
}

pub fn smart_print<'a>(
    cont_ast: ContAstPtr<'a>,
    out: &mut Formatter,
    indent_count: u32,
) -> FmtResult {
    use ContAst::*;

    fn indent(out: &mut Formatter, n: u32) -> FmtResult {
        for _ in 0..(n * 4) {
            out.write_char(' ')?;
        }
        Ok(())
    }

    match cont_ast {
        &Primop {
            ref op,
            ref terminals,
            ref exports,
            ref continuations,
        } => {
            indent(out, indent_count)?;
            writeln!(out, "{:?}({:?}) -> ({:?}) =>", op, terminals, exports)?;
            for cont in continuations {
                smart_print(cont, out, indent_count + 1)?;
            }
        }
        &Fix { ref functions, ref continuation } => {
            for &Function{ref name, ref params, ref body} in functions {
                indent(out, indent_count)?;
                writeln!(out, "fix fn {:?}({:?}) =>", name, params)?;
                smart_print(body, out, indent_count + 1)?;
            }
            indent(out, indent_count + 1)?;
            writeln!(out, "continue with:")?;
            smart_print(continuation, out, indent_count + 2)?;
        }
        &Call {
            ref target,
            ref params,
            ref continuation,
        } => {
            indent(out, indent_count)?;
            write!(out, "call {:?}({:?}) -> {:?}", target, params, continuation)?;
            out.write_char('\n')?;
        }
    }

    Ok(())
}
