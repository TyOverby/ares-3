use vm::*;
use vm;
use self::Instruction::*;
use value::Value::*;
use value::{new_func, AresMap, Function, Symbol, Value};

fn symval(v: &str) -> Value {
    Value::Symbol(Symbol(v.into()))
}

#[test]
fn basic_return_value() {
    let function = new_func(Function {
        name: Some("adder".into()),
        upvars: vec![],
        instructions: vec![Push(Integer(1)), Terminate],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let mut vm = Vm::new();
    assert_eq!(vm.run_function(function), Ok(Integer(1)));
}

#[test]
fn basic_return_value_float() {
    let function = new_func(Function {
        name: Some("adder".into()),
        upvars: vec![],
        instructions: vec![Push(Float(1.234)), Terminate],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let mut vm = Vm::new();

    assert_eq!(vm.run_function(function), Ok(Float(1.234)));
}

#[test]
fn empty_map() {
    let function = new_func(Function {
        name: Some("empty_map".into()),
        upvars: vec![],
        instructions: vec![MapEmpty, Terminate],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let mut vm = Vm::new();
    assert_eq!(vm.run_function(function), Ok(Map(AresMap::new())));
}

#[test]
fn map_with_some_adds() {
    let function = new_func(Function {
        name: Some("empty_map".into()),
        upvars: vec![],
        instructions: vec![
            Push(Value::Integer(20)),
            Push(Value::Integer(5)),
            MapEmpty,
            MapInsert,
            Terminate,
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });
    let mut vm = Vm::new();

    let map = AresMap::new().insert(Value::Integer(20), Value::Integer(5));
    assert_eq!(vm.run_function(function), Ok(Map(map)));
}

#[test]
fn map_get() {
    let function = new_func(Function {
        name: Some("empty_map".into()),
        upvars: vec![],
        instructions: vec![
            Push(Value::Integer(20)),
            Push(Value::Integer(5)),
            MapEmpty,
            MapInsert,
            Push(Value::Integer(20)),
            MapGet,
            Terminate,
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });
    let mut vm = Vm::new();

    assert_eq!(vm.run_function(function), Ok(Integer(5)));
}

#[test]
fn bad_map_get() {
    let function = new_func(Function {
        name: Some("empty_map".into()),
        upvars: vec![],
        instructions: vec![MapEmpty, Push(Value::Integer(20)), MapGet, Terminate],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let mut vm = Vm::new();

    assert_eq!(
        vm.run_function(function),
        Err(VmError::KeyNotFound(Value::Integer(20)))
    );
}

#[test]
fn test_addition() {
    let function = new_func(Function {
        name: Some("adder".into()),
        upvars: vec![],
        instructions: vec![Push(Integer(5)), Push(Integer(10)), Add, Terminate],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });
    let mut vm = Vm::new();
    assert_eq!(vm.run_function(function), Ok(Integer(15)));
}

#[test]
fn test_function_call() {
    let adder = new_func(Function {
        name: Some("adder".into()),
        upvars: vec![],
        instructions: vec![Add, Terminate],
        args_count: 2,
        upvars_count: 0,
        locals_count: 0,
    });

    let main = new_func(Function {
        name: Some("main".into()),
        upvars: vec![],
        instructions: vec![
            Push(Function(adder)),
            Push(Integer(5)),
            Push(Integer(6)),
            Call(2),
            Terminate,
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let mut vm = Vm::new();
    assert_eq!(vm.run_function(main), Ok(Integer(11)));
}

/*
#[test]
fn recursive_fn() {
    let nullfunc = new_func(Function {
        name: Some("NULL".into()),
        upvars: vec![],
        instructions: vec![],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let recursive_infinite = new_func(Function {
        name: Some("recursive infinite".into()),
        upvars: vec![],
        instructions: vec![Push(Function(nullfunc)), Call(0)],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    if let &mut Push(ref mut f) = &mut recursive_infinite.borrow_mut().instructions[0] {
        *f = Function(recursive_infinite.clone());
    }

    let mut vm = Vm::new(recursive_infinite);
    for i in 0..100 {
        vm.step().unwrap();
        vm.step().unwrap();
        //assert_eq!(vm.stack.link_len(), i + 2);
    }
}
*/

#[test]
fn reset_without_a_shift() {
    let inside_reset = new_func(Function {
        name: Some("inside reset".into()),
        upvars: vec![],
        instructions: vec![Push(Integer(1)), Terminate],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let main = new_func(Function {
        name: Some("main".into()),
        upvars: vec![],
        instructions: vec![
            Push(Function(inside_reset)),
            Push(symval("hi")),
            Reset,
            Terminate,
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let mut vm = Vm::new();

    assert_eq!(vm.run_function(main), Ok(Integer(1)));
}

#[test]
fn reset_with_an_id_shift() {
    let id = new_func(Function {
        name: Some("id".into()),
        upvars: vec![],
        instructions: vec![Terminate],
        args_count: 1,
        upvars_count: 0,
        locals_count: 0,
    });

    let reset_closure = new_func(Function {
        name: Some("reset closure".into()),
        upvars: vec![],
        instructions: vec![Push(Function(id)), Push(symval("hi")), Shift, Terminate],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let main = new_func(Function {
        name: Some("main".into()),
        upvars: vec![],
        instructions: vec![
            Push(Function(reset_closure)),
            Push(symval("hi")),
            Reset,
            Terminate,
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });
    let mut vm = Vm::new();

    let v = vm.run_function(main).unwrap();
    assert!(v.is_continuation());

    let new_main = new_func(Function {
        name: Some("main2".into()),
        instructions: vec![Push(v), Push(Integer(5)), Resume, Terminate],
        upvars: vec![],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });
    assert_eq!(vm.run_function(new_main), Ok(Integer(5)));
}

#[test]
fn reset_using_shift_expression() {
    let id = new_func(Function {
        name: Some("id".into()),
        instructions: vec![Terminate],
        upvars: vec![],
        args_count: 1,
        upvars_count: 0,
        locals_count: 0,
    });

    let reset_closure = new_func(Function {
        name: Some("reset closure".into()),
        upvars: vec![],
        instructions: vec![
            Push(Integer(10)),
            Push(Function(id)),
            Push(symval("hi")),
            Shift,
            Add,
            Terminate,
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let main = new_func(Function {
        name: Some("main2".into()),
        upvars: vec![],
        instructions: vec![
            Push(Function(reset_closure)),
            Push(symval("hi")),
            Reset,
            Push(Integer(5)),
            Resume,
            Terminate,
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });
    let mut vm = Vm::new();
    assert_eq!(vm.run_function(main), Ok(Integer(15)));
}


#[test]
fn setting_module_variables() {
    let main = new_func(Function {
        name: Some("main2".into()),
        upvars: vec![],
        instructions: vec![
            Push(Integer(5)),
            Push(symval("variable_name")),
            Push(symval("module_name")),
            ModuleAdd,
            Push(symval("variable_name")),
            Push(symval("module_name")),
            ModuleGet,
            Terminate,
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });
    let mut vm = Vm::new();
    assert_eq!(vm.run_function(main), Ok(Integer(5)));
}
