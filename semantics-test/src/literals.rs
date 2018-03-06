#[allow(unused_imports)]
use super::*;

#[test]
fn integers() {
    let out = run("debug(10);");
    assert_eq!(out, vec![Value::Integer(10)]);
}

#[test]
fn floats() {
    let out = run("debug(1.23);");
    assert_eq!(out, vec![Value::Float(1.23)]);
}
