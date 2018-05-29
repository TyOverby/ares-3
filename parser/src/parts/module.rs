use *;

pub fn parse_module<'a>(
    mut tokens: &'a [Token<'a>],
    module_id: &'a str,
    alloc: &mut Allocator<'a>,
) -> Result<'a> {
    let mut statements = vec![];

    while !tokens.is_empty() {
        let (statement, tokens_n) = parse_statement(tokens, alloc)?;
        tokens = tokens_n;
        statements.push(statement);
    }

    let statements = alloc.alloc_iter(statements);
    return Ok((
        alloc.alloc(Ast::Module {
            statements,
            module_id,
        }),
        tokens,
    ));
}

#[test]
fn single_function_call_statement_module() {
    use test_util::with_parsed_module;

    with_parsed_module("abc();", "module-name", |res| {
        assert!(res.is_ok());
    });
}

#[test]
fn multiple_function_call_statement_module() {
    use test_util::with_parsed_module;

    with_parsed_module("abc(); def();", "module-name", |res| {
        let (res, _) = res.unwrap();
        matches!(res, &Ast::Module{ref statements, module_id: "module-name"}, statements.len() == 2);
    });
}

#[test]
fn single_let_decl() {
    use test_util::with_parsed_module;

    with_parsed_module("let x = 10;", "module-name", |res| {
        let (res, _) = res.unwrap();
        matches!(res, &Ast::Module{ref statements, module_id: "module-name"}, statements.len() == 1);
    });
}

#[test]
fn multiple_let_decl() {
    use test_util::with_parsed_module;

    with_parsed_module("let x = 10; let y = 20;", "module-name", |res| {
        let (res, _) = res.unwrap();
        matches!(res, &Ast::Module{ref statements, module_id: "module-name"}, statements.len() == 2);
    });
}
