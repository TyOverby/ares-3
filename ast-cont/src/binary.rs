use super::*;

pub fn do_binary<'c>(
    l: &'c Ast<'c>,
    r: &'c Ast<'c>,
    c: WithContinue<'c>,
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
