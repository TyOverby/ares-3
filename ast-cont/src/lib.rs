#![feature(fnbox)]

extern crate lexer;
extern crate parser;
extern crate typed_arena;

#[cfg(test)]
mod test;

use parser::Ast;
use std::boxed::FnBox;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter, Result as FmtResult, Write};
use typed_arena::Arena;

pub type ContAstPtr<'parse> = &'parse ContAst<'parse>;

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

#[derive(PartialEq)]
pub enum ContAst<'parse> {
    Primop {
        op: PrimOpKind,
        terminals: Vec<Terminal<'parse>>,
        exports: Vec<Ident<'parse>>,
        continuations: Vec<ContAstPtr<'parse>>,
    },
}

pub fn translate<'c>(
    ast: &'c Ast<'c>,
    c: Box<FnBox(Terminal<'c>) -> ContAstPtr<'c> + 'c>,
    idg: &'c IdGet,
    arena: &'c Arena<ContAst<'c>>,
) -> ContAstPtr<'c> {
    match ast {
        &Ast::Add(l, r) => do_binary(l, r, c, PrimOpKind::Add, idg, arena),
        &Ast::Mul(l, r) => do_binary(l, r, c, PrimOpKind::Mul, idg, arena),
        &Ast::Sub(l, r) => do_binary(l, r, c, PrimOpKind::Sub, idg, arena),
        &Ast::Div(l, r) => do_binary(l, r, c, PrimOpKind::Div, idg, arena),
        &Ast::Integer(_, i) => c(Terminal::Integer(i)),
        &Ast::Float(_, f) => c(Terminal::Float(f)),
        &Ast::Identifier(_, s) => c(Terminal::Ident(Ident::Identifier(s))),
        _ => unimplemented!(),
    }
}

fn do_binary<'c>(
    l: &'c Ast,
    r: &'c Ast,
    c: Box<FnBox(Terminal<'c>) -> ContAstPtr<'c> + 'c>,
    op: PrimOpKind,
    idg: &'c IdGet,
    arena: &'c Arena<ContAst<'c>>,
) -> ContAstPtr<'c> {
    let id = idg.get();
    translate(
        l,
        Box::new(move |lv: Terminal<'c>| {
            translate(
                r,
                Box::new(move |rv: Terminal<'c>| {
                    arena.alloc(ContAst::Primop {
                        op: op,
                        terminals: vec![lv, rv],
                        exports: vec![id],
                        continuations: vec![c(Terminal::Ident(id))],
                    }) as &_
                }),
                idg,
                arena,
            )
        }),
        idg,
        arena,
    )
}

impl IdGet {
    fn new() -> IdGet {
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
        for _ in 0..n {
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
            write!(out, "{:?}({:?}) -> ({:?}) => ", op, terminals, exports)?;
            out.write_char('\n')?;
            for cont in continuations {
                smart_print(cont, out, indent_count + 4)?;
            }
        }
    }

    Ok(())
}
