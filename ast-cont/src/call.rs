use super::*;

pub fn build_cont<'c>(
    body: WithContinue<'c>,
    c: WithContinue<'c>,
    idg: &'c IdGet,
    arena: &'c Arena<ContAst<'c>>,
) -> ContAstPtr<'c> {
    let function_id = idg.get();
    let cont_value_id = idg.get();
    arena.alloc(ContAst::Fix {
        functions: vec![Function {
            name: function_id,
            params: vec![cont_value_id],
            body: body(Terminal::Ident(cont_value_id)),
        }],
        continuation: c(Terminal::Ident(function_id)),
    })
}

type WithContinueParams<'c> = Box<FnBox(Vec<Terminal<'c>>) -> ContAstPtr<'c> + 'c>;

pub fn eval_params<'c>(
    args: &'c [&'c Ast<'c>],
    mut params: Vec<Terminal<'c>>,
    c: WithContinueParams<'c>,
    idg: &'c IdGet,
    arena: &'c Arena<ContAst<'c>>,
) -> ContAstPtr<'c> {
    if args.is_empty() {
        return c(params);
    }

    let (first, rest) = (&args[0], &args[1..]);
    translate(
        first,
        Box::new(move |term| {
            params.push(term);
            eval_params(rest, params, c, idg, arena)
        }),
        idg,
        arena,
    )
}

pub fn do_call<'c>(
    target: &'c Ast<'c>,
    args: &'c [&'c Ast<'c>],
    c: WithContinue<'c>,
    idg: &'c IdGet,
    arena: &'c Arena<ContAst<'c>>,
) -> ContAstPtr<'c> {
    build_cont(
        c,
        Box::new(move |cont_term| {
            translate(
                target,
                Box::new(move |target| {
                    eval_params(args, vec![], Box::new(move |params| arena.alloc(ContAst::Call {
                        target: target,
                        params: params,
                        continuation: cont_term
                    }) as &_), idg, arena)
                }),
                idg,
                arena,
            )
        }),
        idg,
        arena,
    )
}
