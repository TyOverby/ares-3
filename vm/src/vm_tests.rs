use vm::*;
use vm;
use self::Instruction::*;
use value::Value::*;
use value::{new_func, AresMap, Function, Value};

fn symval(v: &str) -> Value {
    Value::Symbol(vm::Symbol(v.into()))
}

#[test]
fn basic_return_value() {
    let function = new_func(Function {
        name: Some("adder".into()),
        upvars: vec![],
        instructions: vec![Push(Integer(1)), Ret],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let mut vm = Vm::new(function);
    let mut vm2 = vm.clone();

    assert_eq!(vm.step(), Ok(StepResult::Continue));
    assert_eq!(vm.step(), Ok(StepResult::Done(Integer(1))));

    assert_eq!(vm2.run(), Ok(Integer(1)));
}

#[test]
fn empty_map() {
    let function = new_func(Function {
        name: Some("empty_map".into()),
        upvars: vec![],
        instructions: vec![MapEmpty, Ret],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let mut vm = Vm::new(function);
    assert_eq!(vm.run(), Ok(Map(AresMap::new())));
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
            Ret,
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let mut vm = Vm::new(function);
    let map = AresMap::new().insert(Value::Integer(20), Value::Integer(5));
    assert_eq!(vm.run(), Ok(Map(map)));
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
            Ret,
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let mut vm = Vm::new(function);
    assert_eq!(vm.run(), Ok(Integer(5)));
}

#[test]
fn bad_map_get() {
    let function = new_func(Function {
        name: Some("empty_map".into()),
        upvars: vec![],
        instructions: vec![MapEmpty, Push(Value::Integer(20)), MapGet, Ret],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let mut vm = Vm::new(function);
    assert_eq!(vm.run(), Err(VmError::KeyNotFound(Value::Integer(20))));
}

#[test]
fn test_addition() {
    let function = new_func(Function {
        name: Some("adder".into()),
        upvars: vec![],
        instructions: vec![Push(Integer(5)), Push(Integer(10)), Add, Ret],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let mut vm = Vm::new(function);

    assert_eq!(vm.run(), Ok(Integer(15)));
}

#[test]
fn test_function_call() {
    let adder = new_func(Function {
        name: Some("adder".into()),
        upvars: vec![],
        instructions: vec![Add, Ret],
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
            Ret,
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let mut vm = Vm::new(main);
    assert_eq!(vm.run(), Ok(Integer(11)));
}

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
        assert_eq!(vm.stack.link_len(), i + 2);
    }
}

#[test]
fn reset_without_a_shift() {
    let inside_reset = new_func(Function {
        name: Some("inside reset".into()),
        upvars: vec![],
        instructions: vec![Push(Integer(1)), Ret],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let main = new_func(Function {
        name: Some("main".into()),
        upvars: vec![],
        instructions: vec![Push(Function(inside_reset)), Push(symval("hi")), Reset, Ret],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let mut vm = Vm::new(main);
    assert_eq!(vm.run(), Ok(Integer(1)));
}

#[test]
fn reset_with_an_id_shift() {
    let id = new_func(Function {
        name: Some("id".into()),
        upvars: vec![],
        instructions: vec![Ret],
        args_count: 1,
        upvars_count: 0,
        locals_count: 0,
    });

    let reset_closure = new_func(Function {
        name: Some("reset closure".into()),
        upvars: vec![],
        instructions: vec![Push(Function(id)), Push(symval("hi")), Shift, Ret],
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
            Ret,
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let mut vm = Vm::new(main);
    let v = vm.run().unwrap();
    assert!(v.is_continuation());

    let new_main = new_func(Function {
        name: Some("main2".into()),
        instructions: vec![Push(v), Push(Integer(5)), Resume, Ret],
        upvars: vec![],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });
    let mut vm = Vm::new(new_main);
    assert_eq!(vm.run(), Ok(Integer(5)));
}

#[test]
fn reset_using_shift_expression() {
    let id = new_func(Function {
        name: Some("id".into()),
        instructions: vec![Ret],
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
            Ret,
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
            Ret,
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let mut vm = Vm::new(main);
    assert_eq!(vm.run(), Ok(Integer(15)));
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
            Ret,
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let mut vm = Vm::new(main);
    assert_eq!(vm.run(), Ok(Integer(5)));
}
