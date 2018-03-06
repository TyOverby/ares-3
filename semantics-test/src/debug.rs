#[allow(unused_imports)]
use super::*;

#[test]
fn basic() {
    let out = run("debug(10);");
    assert_eq!(out, vec![Value::Integer(10)]);
}

#[test]
fn multiple() {
    let out = run("debug(10); debug(20);");
    assert_eq!(out, vec![Value::Integer(10), Value::Integer(20)]);
}
