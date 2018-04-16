use vm::*;
use self::Instruction::*;
use value::Value::*;
use value::{new_func, AresMap, Function, Symbol, Value};
use value::BuiltFunction;

fn symval(v: &str) -> Value {
    Value::Symbol(Symbol(v.into()))
}

#[test]
fn basic_return_value() {
    let function = new_func(Function {
        name: Some("adder".into()),
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
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
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
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
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
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
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
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
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
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
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
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
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
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
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
        instructions: vec![Push(Value::Integer(10)), Resume],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let g = new_func(Function {
        name: Some("g".into()),
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
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
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
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
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
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
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
        instructions: vec![Push(Value::Integer(10)), Terminate],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let g = new_func(Function {
        name: Some("g".into()),
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
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
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
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
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
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
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
        instructions: vec![Push(Integer(11)), Resume],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let printer = new_func(Function {
        name: Some("printer".into()),
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
        instructions: vec![Debug, Push(Integer(9)), Resume],
        args_count: 1,
        upvars_count: 0,
        locals_count: 0,
    });

    let after_get_x = new_func(Function {
        name: Some("after_get_x".into()),
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
        instructions: vec![Push(Value::Integer(1)), Add, Resume],
        args_count: 1,
        upvars_count: 0,
        locals_count: 0,
    });

    let inside_print = new_func(Function {
        name: Some("inside_print".into()),
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
        instructions: vec![
            Push(Value::Function(after_get_x)),
            BuildFunction,
            Push(Value::Function(get_x)),
            BuildFunction,
            Call(0),
        ],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let main = new_func(Function {
        name: Some("main".into()),
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
        instructions: vec![
            Push(Function(printer)),
            BuildFunction,
            Push(Function(inside_print)),
            BuildFunction,
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
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
        instructions: vec![Push(Integer(1)), Terminate],
        args_count: 0,
        upvars_count: 0,
        locals_count: 0,
    });

    let main = new_func(Function {
        name: Some("main".into()),
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
        instructions: vec![
            CurrentContinuation,
            Push(Function(inside_reset)),
            BuildFunction,
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
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
        instructions: vec![
            CurrentContinuation,
            Push(Integer(10)),
            Print,
            GetFromStackPosition(2),
            GetFromStackPosition(1),
            GetFromStackPosition(3),
            Call(1),
        ],
        args_count: 1,
        upvars_count: 0,
        locals_count: 0,
    });

    let after_shift = new_func(Function {
        name: Some("after_shift".into()),
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
        instructions: vec![
            Debug,
            Push(Integer(100)),
            Debug,
            Push(Integer(200)),
            Debug,
            Push(Integer(999)),
            Resume,
        ],
        args_count: 1,
        upvars_count: 0,
        locals_count: 0,
    });


    let reseter = new_func(Function {
        name: Some("resetter".into()),
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
        instructions: vec![
            Push(Function(after_shift)),
            BuildFunction,
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
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
        instructions: vec![Resume],
        args_count: 1,
        upvars_count: 0,
        locals_count: 0,
    });

    let main = new_func(Function {
        name: Some("main".into()),
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
        instructions: vec![
            Push(Function(after_reset)),
            BuildFunction,
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

    assert_eq!(res, Ok(Integer(999)));
    assert_eq!(
        vm.debug_values,
        vec![Integer(10), Integer(100), Integer(200)]
    );
}

#[test]
fn reset_and_shift_with_ignored_cont() {
    let shifter = new_func(Function {
        name: Some("shifter".into()),
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
        instructions: vec![Push(Integer(10)), Resume],
        args_count: 1,
        upvars_count: 0,
        locals_count: 0,
    });

    let after_shift = new_func(Function {
        name: Some("after_shift".into()),
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
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
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
        instructions: vec![
            Push(Function(after_shift)),
            BuildFunction,
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
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
        instructions: vec![Resume],
        args_count: 1,
        upvars_count: 0,
        locals_count: 0,
    });

    let main = new_func(Function {
        name: Some("main".into()),
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
        instructions: vec![
            Push(Function(after_reset)),
            BuildFunction,
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
        built: BuiltFunction {
            upvars: vec![],
            continuation: None,
        },
        is_built: false,
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
