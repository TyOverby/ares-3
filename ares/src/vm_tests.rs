use vm::*;
use vm;
use self::Instruction::*;
use value::Value::*;
use value::Value;
use function::{self, new_func};

fn symval(v: &'static str) -> Value {
    Value::Symbol(vm::Symbol(v))
}

#[test]
fn basic_return_value() {
    let function = new_func(function::Function {
        name: Some("adder".into()),
        arg_count: 0,
        instructions: vec![Push(Integer(1)), Ret],
    });

    let mut vm = Vm::new(function);
    let mut vm2 = vm.clone();

    assert_eq!(vm.step(), Ok(StepResult::Continue));
    assert_eq!(vm.step(), Ok(StepResult::Done(Integer(1))));

    assert_eq!(vm2.run(), Ok(Integer(1)));
}

#[test]
fn test_addition() {
    let function = new_func(function::Function {
        name: Some("adder".into()),
        arg_count: 0,
        instructions: vec![Push(Integer(5)), Push(Integer(10)), Add, Ret],
    });

    let mut vm = Vm::new(function);

    assert_eq!(vm.run(), Ok(Integer(15)));
}

#[test]
fn test_function_call() {
    let adder = new_func(function::Function {
        name: Some("adder".into()),
        arg_count: 2,
        instructions: vec![Add, Ret],
    });

    let main = new_func(function::Function {
        name: Some("main".into()),
        arg_count: 0,
        instructions: vec![
            Push(Integer(5)),
            Push(Integer(6)),
            Push(Function(adder)),
            Push(Integer(2)),
            Call,
            Ret,
        ],
    });

    let mut vm = Vm::new(main);
    assert_eq!(vm.run(), Ok(Integer(11)));
}

#[test]
fn recursive_fn() {
    let nullfunc = new_func(function::Function {
        name: Some("NULL".into()),
        arg_count: 0,
        instructions: vec![],
    });

    let recursive_infinite = new_func(function::Function {
        name: Some("recursive infinite".into()),
        arg_count: 0,
        instructions: vec![
            Push(Function(nullfunc)),
            Push(Integer(0)),
            Call
        ],
    });

    if let &mut Push(ref mut f) = &mut recursive_infinite.borrow_mut().instructions[0] {
        *f = Function(recursive_infinite.clone());
    }

    let mut vm = Vm::new(recursive_infinite);
    for i in 0 .. 100 {
        vm.step().unwrap();
        vm.step().unwrap();
        vm.step().unwrap();
        assert_eq!(vm.stack.link_len(), i + 2);
    }
}

#[test]
fn reset_without_a_shift() {
    let inside_reset = new_func(function::Function {
        name: Some("inside reset".into()),
        arg_count: 0,
        instructions: vec![
            Push(Integer(1)),
            Ret
        ]
    });

    let main = new_func(function::Function {
        name: Some("main".into()),
        arg_count: 0,
        instructions: vec![
            Push(Function(inside_reset)),
            Push(symval("hi")),
            Reset,
            Ret
        ]
    });

    let mut vm = Vm::new(main);
    assert_eq!(vm.run(), Ok(Integer(1)));
}

#[test]
fn reset_with_an_id_shift() {
    let id = new_func(function::Function {
        name: Some("id".into()),
        arg_count: 1,
        instructions: vec![Ret]
    });

    let reset_closure = new_func(function::Function {
        name: Some("reset closure".into()),
        arg_count: 0,
        instructions: vec![
            Push(Function(id)),
            Push(symval("hi")),
            Shift,
            Ret,
        ]
    });

    let main = new_func(function::Function {
        name: Some("main".into()),
        arg_count: 0,
        instructions: vec![
            Push(Function(reset_closure)),
            Push(symval("hi")),
            Reset,
            Ret
        ]
    });

    let mut vm = Vm::new(main);
    let v = vm.run();
    if let &Ok(Value::Continuation(_)) = &v { /* good*/ }
    else {
        panic!("not a continuation!: {:?}", v);
    }

    let new_main = new_func(function::Function {
        name: Some("main2".into()),
        arg_count: 0,
        instructions: vec![
            Push(Integer(5)),
            Push(v.unwrap()),
            Resume,
            Ret,
        ]
    });
    let mut vm = Vm::new(new_main);
    assert_eq!(vm.run(), Ok(Integer(5)));
}
