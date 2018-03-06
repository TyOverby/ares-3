#[allow(unused_imports)]
use super::*;

#[test]
fn addition_int_int() {
    let out = run("debug(5 + 2);");
    assert_eq!(out, vec![Value::Integer(7)]);
}

#[test]
fn subtraction_int_int() {
    let out = run("debug(5 - 2);");
    assert_eq!(out, vec![Value::Integer(3)]);
}

#[test]
fn multiplication_int_int() {
    let out = run("debug(5 * 2);");
    assert_eq!(out, vec![Value::Integer(10)]);
}

#[test]
fn division_int_int() {
    let out = run("debug(10 / 2);");
    assert_eq!(out, vec![Value::Integer(5)]);
}

