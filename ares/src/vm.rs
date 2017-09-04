use linked_stack::{LinkedStack, LinkedStackBehavior};
use function::FunctionPtr;
use value::Value;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Symbol(&'static str);

pub type VmResult<T> = Result<T, VmError>;

#[derive(Clone, PartialEq)]
pub(crate) struct StackBehavior;
impl LinkedStackBehavior for StackBehavior {
    type Symbol = Symbol;
    type Error = VmError;

    fn underflow() -> Self::Error {
        VmError::StackUnderflow
    }
    fn overflow() -> Self::Error {
        VmError::StackOverflow
    }
    fn tag_not_found(symbol: Symbol) -> Self::Error {
        VmError::TagNotFound(symbol)
    }
}
pub(crate) type ValueStack = LinkedStack<Value, Symbol, FuncExecData, StackBehavior>;

#[derive(Clone, PartialEq, Debug)]
pub enum VmError {
    StackUnderflow,
    StackOverflow,
    CrossBoundary,
    ArityMismatch { actual: usize, expected: usize },
    TagNotFound(Symbol),
    UnexpectedType(Value),
    RanOutOfInstructions,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Instruction {
    Add,
    Push(Value),
    Dup,
    Print,
    Call,
    Ret,
}

#[derive(Clone, PartialEq)]
pub struct FuncExecData {
    function: FunctionPtr,
    ip: usize,
}

#[derive(PartialEq, Clone)]
pub struct Vm {
    pub(crate) stack: ValueStack,
    pub(crate) debug_values: Vec<Value>,
}

#[derive(PartialEq, Clone, Debug)]
pub enum StepResult {
    Done(Value),
    Continue,
}

impl Vm {
    pub fn new(function: FunctionPtr) -> Vm {
        assert!(function.borrow().arg_count == 0);

        let exec_data = FuncExecData {
            function: function,
            ip: 0,
        };

        Vm {
            stack: ValueStack::new(exec_data),
            debug_values: vec![],
        }
    }

    pub fn run(&mut self) -> VmResult<Value> {
        loop {
            println!("STEP");
            if let StepResult::Done(v) = self.step()? {
                return Ok(v);
            }
        }
    }

    pub fn step(&mut self) -> VmResult<StepResult> {
        use self::Instruction::*;

        let instruction = {
            let &FuncExecData {
                ref function,
                ref ip,
            } = self.stack.aux();
            if *ip >= function.borrow().instructions.len() {
                return Err(VmError::RanOutOfInstructions);
            }
            let instruction = function.borrow().instructions[*ip].clone();

            instruction
        };

        self.stack.aux_mut().ip += 1;

        println!("{:?}", instruction);

        match instruction {
            Add => {
                let l = self.stack.pop()?.to_int()?;
                let r = self.stack.pop()?.to_int()?;
                self.stack.push(Value::Integer(l + r))?;
            }
            Push(v) => {
                self.stack.push(v)?;
            }
            Dup => {
                let v = self.stack.peek()?.clone();
                self.stack.push(v)?;
            }
            Print => {
                let v = self.stack.pop()?;
                self.debug_values.push(v);
            }
            Call => {
                let arg_count = self.stack.pop()?.to_int()?;
                let f = self.stack.pop()?.to_function()?;

                if f.borrow().arg_count != arg_count as usize {
                    return Err(VmError::ArityMismatch {
                        expected: f.borrow().arg_count,
                        actual: arg_count as usize,
                    });
                }

                let args = self.stack.pop_n(arg_count as usize)?;
                self.stack
                    .start_segment(None, FuncExecData { function: f, ip: 0 });
                for arg in args {
                    self.stack.push(arg)?;
                }
            }
            Ret => {
                let retval = self.stack.pop()?;
                if self.stack.link_len() == 1 {
                    return Ok(StepResult::Done(retval));
                } else {
                    self.stack.kill_segment()?;
                    self.stack.push(retval)?;
                }
            }
        }

        Ok(StepResult::Continue)
    }
}

#[test]
fn basic_return_value() {
    use self::Instruction::*;
    use value::Value::*;
    use function::{self, new_func};

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
    use self::Instruction::*;
    use value::Value::*;
    use function::{self, new_func};

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
    use self::Instruction::*;
    use value::Value::*;
    use function::{self, new_func};

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
    use self::Instruction::*;
    use value::Value::*;
    use function::{self, new_func};

    let nullfunc = new_func(function::Function {
        name: Some("NULL".into()),
        arg_count: 0,
        instructions: vec![],
    });

    let recursive_infinite = new_func(function::Function {
        name: Some("recursive infinite".into()),
        arg_count: 0,
        instructions: vec![
            Push(Value::Function(nullfunc)),
            Push(Value::Integer(0)),
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
