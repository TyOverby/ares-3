use value::{new_func, AresMap, ContinuationPtr, Function, FunctionPtr, Symbol, Value, ValueKind};
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use super::resultvec::ResultVec;

pub type VmResult<T> = Result<T, VmError>;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum VmError {
    StackUnderflow,
    StackOverflow,
    CrossBoundary,
    KeyNotFound(Value),
    FieldNotFound(Symbol),
    ArityMismatch { actual: u32, expected: u32 },
    TagNotFound(Symbol),
    UnexpectedType { expected: ValueKind, found: Value },
    RanOutOfInstructions,
    NoModuleDefinition { module: Symbol, definition: Symbol },
    ContinueWithoutContinuation,
}

#[derive(Clone, PartialEq, Debug, PartialOrd, Serialize, Deserialize)]
pub enum Instruction {
    Add,
    Sub,
    Mul,
    Div,

    Push(Value),
    GetFromStackPosition(u32),
    SetToStackPosition(u32),
    Pop,
    Dup,

    Print,
    Debug,

    BuildFunction,
    BuildContinuation,
    Call(u32),
    Terminate,

    InstallContinuation,
    CurrentContinuation,
    Reset,
    Shift,
    Resume,

    ModuleAdd,
    ModuleGet,

    MapEmpty,
    MapInsert,
    MapGet,
}

#[derive(Clone, PartialEq, Debug, PartialOrd, Serialize, Deserialize)]
pub struct FuncExecData {
    function: FunctionPtr,
    ip: usize,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Vm {
    pub debug_values: Vec<Value>,
    pub(crate) modules: HashMap<(Symbol, Symbol), Value>,
}

#[derive(PartialEq, Clone, Debug)]
pub enum StepResult {
    Done(Value),
    Continue,
}

impl Vm {
    pub fn new() -> Vm {
        Vm {
            debug_values: vec![],
            modules: HashMap::new(),
        }
    }

    pub fn run_function(&mut self, fp: FunctionPtr) -> VmResult<Value> {
        let terminate_function = new_func(Function {
            name: Some("<terminate>".into()),
            upvars: vec![],
            instructions: vec![Instruction::Terminate],
            args_count: 1,
            upvars_count: 0,
            locals_count: 0,
        });
        let final_continuation = ContinuationPtr::new(terminate_function, None, None);

        let mut exec_data = FuncExecData {
            function: fp,
            ip: 0,
        };

        let mut function_stack = ResultVec::new();
        let mut continuation = Some(Rc::new(final_continuation));

        loop {
            match self.step(&mut exec_data, &mut function_stack, &mut continuation)? {
                StepResult::Done(v) => return Ok(v),
                StepResult::Continue => continue,
            }
        }
    }

    fn step(
        &mut self,
        func_exec: &mut FuncExecData,
        stack: &mut ResultVec<Value>,
        continuation: &mut Option<Rc<ContinuationPtr>>,
    ) -> VmResult<StepResult> {
        use self::Instruction::*;
        let instruction = {
            let &FuncExecData {
                ref function,
                ref ip,
            } = func_exec as &_;
            if *ip >= function.borrow().instructions.len() {
                return Err(VmError::RanOutOfInstructions);
            }
            function.borrow().instructions[*ip].clone()
        };
        func_exec.ip += 1;

        println!(
            "{}: {:?}",
            func_exec
                .function
                .function
                .borrow()
                .name
                .as_ref()
                .map(AsRef::as_ref)
                .unwrap_or("<unnamed>"),
            instruction
        );

        match instruction {
            Add => {
                let r = stack.pop()?;
                let l = stack.pop()?;
                assert_numeric(&l)?;
                assert_numeric(&r)?;

                let result = match (l, r) {
                    (Value::Integer(l), Value::Integer(r)) => Value::Integer(l + r),
                    (Value::Float(l), Value::Float(r)) => Value::Float(l + r),
                    (Value::Integer(l), Value::Float(r)) => Value::Float(l as f64 + r),
                    (Value::Float(l), Value::Integer(r)) => Value::Float(l + r as f64),
                    _ => unreachable!(),
                };

                stack.push(result)?;
            }
            Sub => {
                let r = stack.pop()?;
                let l = stack.pop()?;
                assert_numeric(&l)?;
                assert_numeric(&r)?;

                let result = match (l, r) {
                    (Value::Integer(l), Value::Integer(r)) => Value::Integer(l - r),
                    (Value::Float(l), Value::Float(r)) => Value::Float(l - r),
                    (Value::Integer(l), Value::Float(r)) => Value::Float(l as f64 - r),
                    (Value::Float(l), Value::Integer(r)) => Value::Float(l - r as f64),
                    _ => unreachable!(),
                };

                stack.push(result)?;
            }
            Mul => {
                let r = stack.pop()?;
                let l = stack.pop()?;
                assert_numeric(&l)?;
                assert_numeric(&r)?;

                let result = match (l, r) {
                    (Value::Integer(l), Value::Integer(r)) => Value::Integer(l * r),
                    (Value::Float(l), Value::Float(r)) => Value::Float(l * r),
                    (Value::Integer(l), Value::Float(r)) => Value::Float(l as f64 * r),
                    (Value::Float(l), Value::Integer(r)) => Value::Float(l * r as f64),
                    _ => unreachable!(),
                };

                stack.push(result)?;
            }
            Div => {
                let r = stack.pop()?;
                let l = stack.pop()?;
                assert_numeric(&l)?;
                assert_numeric(&r)?;

                let result = match (l, r) {
                    (Value::Integer(l), Value::Integer(r)) => Value::Integer(l / r),
                    (Value::Float(l), Value::Float(r)) => Value::Float(l / r),
                    (Value::Integer(l), Value::Float(r)) => Value::Float(l as f64 / r),
                    (Value::Float(l), Value::Integer(r)) => Value::Float(l / r as f64),
                    _ => unreachable!(),
                };

                stack.push(result)?;
            }

            s @ BuildFunction | s @ BuildContinuation => {
                let f = stack.pop()?.into_function()?;
                let mut function = f.borrow().clone();
                assert!(function.upvars.len() == 0);
                let upvars = stack.pop_n(function.upvars_count)?;
                function.upvars = upvars.inner;
                let func = new_func(function);
                match s {
                    BuildFunction => stack.push(Value::Function(func))?,
                    BuildContinuation => stack.push(Value::Continuation(
                        ContinuationPtr::new(func, continuation.clone(), None),
                    ))?,
                    _ => unreachable!(),
                }
            }
            GetFromStackPosition(pos) => {
                let v = stack.get(pos)?.clone();
                stack.push(v)?;
            }
            SetToStackPosition(pos) => {
                let value = stack.pop()?;
                stack.set(pos, value)?;
            }
            Push(v) => {
                stack.push(v)?;
            }
            Pop => {
                stack.pop()?;
            }

            ModuleAdd => {
                let module_name = stack.pop()?.into_symbol()?;
                let definition_name = stack.pop()?.into_symbol()?;
                let value = stack.pop()?;
                self.modules.insert((module_name, definition_name), value);
            }

            ModuleGet => {
                let module_name = stack.pop()?.into_symbol()?;
                let definition_name = stack.pop()?.into_symbol()?;
                let value = self.modules
                    .get(&(module_name.clone(), definition_name.clone()));

                let value = value.ok_or_else(|| {
                    VmError::NoModuleDefinition {
                        module: module_name,
                        definition: definition_name,
                    }
                })?;

                stack.push(value.clone())?;
            }

            MapEmpty => {
                stack.push(Value::Map(AresMap::new()))?;
            }
            MapInsert => {
                let map = stack.pop()?.into_map()?;
                let v = stack.pop()?;
                let k = stack.pop()?;
                let map = map.insert(k, v);
                stack.push(Value::Map(map))?;
            }
            MapGet => {
                let k = stack.pop()?;
                let map = stack.pop()?.into_map()?;
                if let Some(v) = map.get(&k) {
                    stack.push(v.clone())?;
                } else {
                    return Err(VmError::KeyNotFound(k));
                }
            }
            Dup => {
                let v = stack.peek()?.clone();
                stack.push(v)?;
            }
            Print => {
                println!("{:?}", stack);
            }
            Debug => {
                self.debug_values.push(stack.pop()?);
            }
            Call(arg_count) => {
                let args = stack.pop_n(arg_count)?;
                let f = stack.pop()?.into_function()?;
                let mut c = stack.pop()?.into_continuation()?;
                c.parent = continuation.take();
                *continuation = Some(Rc::new(c));
                setup_new_function(f, args, stack, func_exec)?;
            }
            Terminate => {
                let result = stack.pop()?;
                return Ok(StepResult::Done(result));
            }
            InstallContinuation => {
                let c = stack.pop()?.into_continuation()?;
                *continuation = Some(Rc::new(join_cont_chain(continuation.take(), c)));
            }
            CurrentContinuation => {
                if continuation.is_none() {
                    return Err(VmError::ContinueWithoutContinuation);
                }
                stack.push(Value::Continuation(
                    (continuation.as_ref().unwrap().deref()).clone(),
                ))?;
            }
            Reset => {
                let tag = stack.pop()?.into_symbol()?;
                let function = stack.pop()?.into_function()?;
                let mut after_reset = stack.pop()?.into_continuation()?;
                *continuation = Some(Rc::new(join_cont_chain(
                    Some(Rc::new(join_cont_chain(continuation.take(), after_reset))),
                    ContinuationPtr::new(continue_up(), None, Some(tag)),
                )));
                setup_new_function(function, ResultVec::new(), stack, func_exec)?;
            }
            Shift => {
                let tag = stack.pop()?.into_symbol()?;
                let function = stack.pop()?.into_function()?;
                let mut after_shift = stack.pop()?.into_continuation()?;

                let (upper, lower) = split_cont_chain(tag.clone(), continuation.take());
                *continuation = upper;

                if lower.is_none() {
                    return Err(VmError::TagNotFound(tag));
                }

                after_shift.parent = lower;
                setup_new_function(
                    function,
                    ResultVec::new_with(vec![Value::Continuation(after_shift)]),
                    stack,
                    func_exec,
                )?;
            }
            Resume => {
                if continuation.is_none() {
                    return Err(VmError::ContinueWithoutContinuation);
                }

                let cc: ContinuationPtr = (*continuation.take().unwrap()).clone();
                let ContinuationPtr {
                    function, parent, ..
                } = cc;
                *continuation = parent;
                assert!(function.function.borrow().args_count == 1);

                let continue_with_value = stack.pop()?;
                setup_new_function(
                    function,
                    ResultVec::new_with(vec![continue_with_value]),
                    stack,
                    func_exec,
                )?;
            }
        }

        Ok(StepResult::Continue)
    }
}

fn rc_get<T: Clone>(rc: Rc<T>) -> T {
    match Rc::try_unwrap(rc) {
        Ok(t) => t,
        Err(rc) => rc.deref().clone(),
    }
}

fn join_cont_chain(
    left: Option<Rc<ContinuationPtr>>,
    mut right: ContinuationPtr,
) -> ContinuationPtr {
    match right.parent.map(rc_get) {
        None => {
            right.parent = left;
            return right;
        }
        Some(p) => {
            right.parent = Some(Rc::new(join_cont_chain(left, p)));
            return right;
        }
    }
}


fn split_cont_chain(
    tag: Symbol,
    current: Option<Rc<ContinuationPtr>>,
) -> (Option<Rc<ContinuationPtr>>, Option<Rc<ContinuationPtr>>) {
    if current.is_none() {
        return (None, None);
    }
    let mut current = rc_get(current.unwrap());

    if current.tag.as_ref() == Some(&tag) {
        let cp = current.parent.take();
        current.parent = None;
        (cp, Some(Rc::new(current)))
    } else {
        let (high, low) = split_cont_chain(tag, current.parent.clone());
        current.parent = low;
        (high, Some(Rc::new(current)))
    }
}

fn setup_new_function(
    f: FunctionPtr,
    args: ResultVec<Value>,
    stack: &mut ResultVec<Value>,
    func_exec: &mut FuncExecData,
) -> VmResult<()> {
    if f.borrow().args_count != args.inner.len() as u32 {
        return Err(VmError::ArityMismatch {
            expected: f.borrow().args_count,
            actual: args.inner.len() as u32,
        });
    }

    let locals_count = f.borrow().locals_count;
    let upvars = f.borrow().upvars.clone();

    let exec_data = FuncExecData {
        function: f.clone(),
        ip: 0,
    };

    *stack = ResultVec::new();
    stack.push(Value::Function(f))?;
    *func_exec = exec_data;

    for arg in args.inner {
        stack.push(arg)?;
    }

    for upvar in upvars {
        stack.push(upvar.clone())?;
    }

    for _ in 0..locals_count {
        stack.push(Value::Integer(9999999999))?;
    }
    Ok(())
}


fn assert_numeric(v: &Value) -> VmResult<()> {
    if v.kind() != ValueKind::Integer && v.kind() != ValueKind::Float {
        return Err(VmError::UnexpectedType {
            expected: ValueKind::Integer,
            found: v.clone(),
        });
    }
    return Ok(());
}

fn continue_up() -> FunctionPtr {
    new_func(Function {
        name: Some("<continue-shim>".into()),
        upvars: vec![],
        instructions: vec![Instruction::Resume],
        args_count: 1,
        upvars_count: 0,
        locals_count: 0,
    })
}
