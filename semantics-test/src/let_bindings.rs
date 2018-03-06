#[allow(unused_imports)]
use super::*;

#[test]
fn basic() {
    let out = run("let x = 10; debug(x);");
    assert_eq!(out, vec![Value::Integer(10)]);
}

#[test]
fn in_function_block() {
    let out = run("let f() = {let x = 10; x}; debug(f());");
    assert_eq!(out, vec![Value::Integer(10)]);
}

#[test]
fn shadowing_in_module_scope() {
    let out = run(
        r#"
    let x = 10;
    let f() = {debug(x); 0};
    let x = 20;
    let g() = {debug(x); 0};
    f();
    g();"#,
    );
    assert_eq!(out, vec![Value::Integer(10), Value::Integer(20)]);
}

/*
#[test]
fn shadowing_in_function_scope() {
    let out = run(
        r#"
    let ff() = {
        let x = 10;
        let f() = {debug(x); 0};
        let x = 20;
        let g() = {debug(x); 0};
        f();
        g();
        0
    };
    ff();
    "#,
    );
    assert_eq!(out, vec![Value::Integer(10), Value::Integer(20)]);
}
*/
