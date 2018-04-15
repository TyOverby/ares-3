use vm::*;
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
fn test_tail_call() {
    let last = new_func(Function {
        name: Some("last".into()),
        upvars: vec![],
        instructions: vec![Push(Value::Integer(10)), Resume],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let g = new_func(Function {
        name: Some("g".into()),
        upvars: vec![],
        instructions: vec![
            CurrentContinuation,
            Push(Value::Function(last)),
            BuildFunction,
            Call(0),
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let f = new_func(Function {
        name: Some("f".into()),
        upvars: vec![],
        instructions: vec![
            CurrentContinuation,
            Push(Value::Function(g)),
            BuildFunction,
            Call(0),
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let main = new_func(Function {
        name: Some("main".into()),
        upvars: vec![],
        instructions: vec![
            CurrentContinuation,
            Push(Function(f)),
            BuildFunction,
            Call(0),
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let mut vm = Vm::new();
    assert_eq!(vm.run_function(main), Ok(Integer(10)));
}

#[test]
fn fake_test_tail_call() {
    let last = new_func(Function {
        name: Some("last".into()),
        upvars: vec![],
        instructions: vec![Push(Value::Integer(10)), Terminate],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let g = new_func(Function {
        name: Some("g".into()),
        upvars: vec![],
        instructions: vec![
            CurrentContinuation,
            Push(Value::Function(last)),
            BuildFunction,
            Call(0),
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let f = new_func(Function {
        name: Some("f".into()),
        upvars: vec![],
        instructions: vec![
            CurrentContinuation,
            Push(Value::Function(g)),
            BuildFunction,
            Call(0),
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let main = new_func(Function {
        name: Some("main".into()),
        upvars: vec![],
        instructions: vec![
            CurrentContinuation,
            Push(Function(f)),
            BuildFunction,
            Call(0),
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let mut vm = Vm::new();
    assert_eq!(vm.run_function(main), Ok(Integer(10)));
}

#[test]
fn test_function_call() {
    let get_x = new_func(Function {
        name: Some("getX".into()),
        upvars: vec![],
        instructions: vec![Push(Integer(11)), Resume],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let printer = new_func(Function {
        name: Some("printer".into()),
        upvars: vec![],
        instructions: vec![Debug, Push(Integer(9)), Resume],
        args_count: 1,
        upvars_count: 0,
        locals_count: 0,
    });

    let after_get_x = new_func(Function {
        name: Some("after_get_x".into()),
        upvars: vec![],
        instructions: vec![Push(Value::Integer(1)), Add, Resume],
        args_count: 1,
        upvars_count: 0,
        locals_count: 0,
    });

    let inside_print = new_func(Function {
        name: Some("inside_print".into()),
        upvars: vec![],
        instructions: vec![
            Push(Value::Function(after_get_x)),
            BuildContinuation,
            Push(Value::Function(get_x)),
            Call(0),
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let main = new_func(Function {
        name: Some("main".into()),
        upvars: vec![],
        instructions: vec![
            Push(Function(printer)),
            BuildContinuation,
            Push(Function(inside_print)),
            Call(0),
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let mut vm = Vm::new();
    let result = vm.run_function(main).unwrap();
    assert_eq!(vm.debug_values, vec![Integer(12)]);
    assert_eq!(result, Integer(9));
}

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
            CurrentContinuation,
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
fn reset_and_shift_with_called_cont() {
    let shifter = new_func(Function {
        name: Some("shifter".into()),
        upvars: vec![],
        instructions: vec![Push(Integer(10)), Resume],
        args_count: 1,
        upvars_count: 0,
        locals_count: 0,
    });

    let after_shift = new_func(Function {
        name: Some("after_shift".into()),
        upvars: vec![],
        instructions: vec![
            Push(Integer(100)),
            Debug,
            Push(Integer(100)),
            Debug,
            Push(Integer(999)),
            Resume,
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });


    let reseter = new_func(Function {
        name: Some("resetter".into()),
        upvars: vec![],
        instructions: vec![
            Push(Function(after_shift)),
            BuildContinuation,
            Push(Function(shifter)),
            BuildFunction,
            Push(symval("io")),
            Shift,
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let after_reset = new_func(Function {
        name: Some("after_reset".into()),
        upvars: vec![],
        instructions: vec![Resume],
        args_count: 1,
        upvars_count: 0,
        locals_count: 0,
    });

    let main = new_func(Function {
        name: Some("main".into()),
        upvars: vec![],
        instructions: vec![
            Push(Function(after_reset)),
            BuildContinuation,
            Push(Function(reseter)),
            BuildFunction,
            Push(symval("io")),
            Reset,
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });
    let mut vm = Vm::new();
    let res = vm.run_function(main);

    assert_eq!(res, Ok(Integer(10)));
    assert_eq!(vm.debug_values, vec![]);
}

#[test]
fn reset_and_shift_with_ignored_cont() {
    let shifter = new_func(Function {
        name: Some("shifter".into()),
        upvars: vec![],
        instructions: vec![Push(Integer(10)), Resume],
        args_count: 1,
        upvars_count: 0,
        locals_count: 0,
    });

    let after_shift = new_func(Function {
        name: Some("after_shift".into()),
        upvars: vec![],
        instructions: vec![
            Push(Integer(100)),
            Debug,
            Push(Integer(100)),
            Debug,
            Push(Integer(999)),
            Resume,
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });


    let reseter = new_func(Function {
        name: Some("resetter".into()),
        upvars: vec![],
        instructions: vec![
            Push(Function(after_shift)),
            BuildContinuation,
            Push(Function(shifter)),
            BuildFunction,
            Push(symval("io")),
            Shift,
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let after_reset = new_func(Function {
        name: Some("after_reset".into()),
        upvars: vec![],
        instructions: vec![Resume],
        args_count: 1,
        upvars_count: 0,
        locals_count: 0,
    });

    let main = new_func(Function {
        name: Some("main".into()),
        upvars: vec![],
        instructions: vec![
            Push(Function(after_reset)),
            BuildContinuation,
            Push(Function(reseter)),
            BuildFunction,
            Push(symval("io")),
            Reset,
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });
    let mut vm = Vm::new();
    let res = vm.run_function(main);

    assert_eq!(res, Ok(Integer(10)));
    assert_eq!(vm.debug_values, vec![]);
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
