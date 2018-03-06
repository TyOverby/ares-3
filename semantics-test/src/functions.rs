#[allow(unused_imports)]
use super::*;

#[test]
fn no_params() {
    let out = run("let f() = 2; debug(f());");
    assert_eq!(out, vec![Value::Integer(2)]);
}

#[test]
fn use_of_parameters() {
    let out = run(r#"let f(x, y) = x + y; debug(f(1, 2));"#);
    assert_eq!(out, vec![Value::Integer(3)]);
}

#[test]
fn function_returning_function() {
    let out = run(r#"let f() = {let g() = 10; g}; debug(f()());"#);
    assert_eq!(out, vec![Value::Integer(10)]);
}
