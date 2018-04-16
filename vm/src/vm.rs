use value::{new_func, AresMap, BuiltFunction, Function, FunctionPtr, Symbol, Value, ValueKind};
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
    CallOnUnbuiltFunction,
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
    Swap,
    Pop,
    Dup,

    Print,
    Debug,

    BuildFunction,
    Call(u32),
    Terminate,

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
            built: BuiltFunction {
                upvars: vec![],
                continuation: None,
            },
            is_built: true,
            instructions: vec![Instruction::Terminate],
            args_count: 1,
            upvars_count: 0,
            locals_count: 0,
        });

        let mut fp = rc_get(fp.function).clone();
        assert_eq!(fp.args_count, 0);
        assert_eq!(fp.upvars_count, 0);

        fp.built = BuiltFunction {
            upvars: vec![],
            continuation: Some((terminate_function, None)),
        };
        fp.is_built = true;

        let fp = new_func(fp);

        let mut exec_data = FuncExecData {
            function: fp,
            ip: 0,
        };

        let mut function_stack = ResultVec::new();

        loop {
            match self.step(&mut exec_data, &mut function_stack)? {
                StepResult::Done(v) => return Ok(v),
                StepResult::Continue => continue,
            }
        }
    }

    fn step(
        &mut self,
        func_exec: &mut FuncExecData,
        stack: &mut ResultVec<Value>,
    ) -> VmResult<StepResult> {
        use self::Instruction::*;
        let instruction = {
            let &FuncExecData {
                ref function,
                ref ip,
            } = func_exec as &_;
            if *ip >= function.instructions.len() {
                return Err(VmError::RanOutOfInstructions);
            }
            function.instructions[*ip].clone()
        };
        func_exec.ip += 1;

        println!(
            "{}: {:?}",
            func_exec
                .function
                .function
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

            BuildFunction => {
                let function = stack.pop()?.into_function()?;
                let mut function = rc_get(function.function);
                assert!(!function.is_built);
                let upvars = stack.pop_n(function.upvars_count)?;
                function.built = BuiltFunction {
                    upvars: upvars.inner,
                    continuation: None,
                };
                function.is_built = true;
                let function = new_func(function);
                stack.push(Value::Function(function))?;
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
            Swap => {
                let a = stack.pop()?;
                let b = stack.pop()?;
                stack.push(a)?;
                stack.push(b)?;
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
            Terminate => {
                let result = stack.pop()?;
                return Ok(StepResult::Done(result));
            }
            CurrentContinuation => {
                let cc = func_exec.function.continuation();
                assert!(cc.is_some());
                stack.push(Value::Function(cc.unwrap().0))?;
            }
            Call(arg_count) => {
                let args = stack.pop_n(arg_count)?;
                let function = stack.pop()?.into_function()?;
                let continuation = stack.pop()?.into_function()?;
                assert!(function.is_built);
                assert!(continuation.is_built);

                let (function, _) = join_cont_chain(
                    func_exec.function.continuation(),
                    join_cont_chain(Some((continuation, None)), (function, None)),
                );
                assert!(function.continuation().is_some());

                setup_new_function(function, args, stack, func_exec)?;
            }
            Reset => {
                let tag = stack.pop()?.into_symbol()?;
                let function = stack.pop()?.into_function()?;
                let after_reset = stack.pop()?.into_function()?;
                assert!(function.is_built);
                assert!(function.args_count == 0);
                assert!(after_reset.is_built);

                // current continuation <- after_reset <- continue_up('s) <- function
                let function = join_cont_chain(
                    Some(join_cont_chain(
                        func_exec.function.continuation(),
                        (continue_up((after_reset, Some(tag))), None),
                    )),
                    (function, None),
                );

                setup_new_function(function.0, ResultVec::new(), stack, func_exec)?;
            }
            Shift => {
                let tag = stack.pop()?.into_symbol()?;
                let function = stack.pop()?.into_function()?;
                let after_shift = stack.pop()?.into_function()?;

                assert!(function.args_count == 1);

                let cc = func_exec.function.continuation();
                let (high, low) = split_cont_chain(tag.clone(), cc);
                let (high, low) = match (high, low) {
                    (None, _) => return Err(VmError::TagNotFound(tag)),
                    (_, None) => panic!(),
                    (Some(h), Some(l)) => (h, l),
                };

                let function = function.with_continuation(high);
                let continuation_parameter = join_cont_chain(Some(low), (after_shift, None));

                setup_new_function(
                    function,
                    ResultVec::new_with(vec![Value::Function(continuation_parameter.0)]),
                    stack,
                    func_exec,
                )?;
            }
            Resume => {
                let function = match func_exec.function.continuation() {
                    Some(f) => f.0,
                    None => return Err(VmError::ContinueWithoutContinuation),
                };

                assert!(function.function.args_count == 1);

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

type ContPair = (FunctionPtr, Option<Symbol>);
fn join_cont_chain(left: Option<ContPair>, right: ContPair) -> ContPair {
    match right.0.continuation() {
        None => (right.0.with_opt_continuation(left), right.1),
        Some(p) => {
            let px = join_cont_chain(left, p);
            (right.0.with_opt_continuation(Some(px)), right.1)
        }
    }
}


fn split_cont_chain(
    tag: Symbol,
    current: Option<(FunctionPtr, Option<Symbol>)>,
) -> (
    Option<(FunctionPtr, Option<Symbol>)>,
    Option<(FunctionPtr, Option<Symbol>)>,
) {
    if current.is_none() {
        return (None, None);
    }
    let (current_fn, current_tag) = current.unwrap();
    let mut current_fn = rc_get(current_fn.function);

    if current_tag.as_ref() == Some(&tag) {
        let cp = current_fn.built.continuation.take();
        current_fn.built.continuation = None;
        (
            cp,
            Some((
                FunctionPtr {
                    function: Rc::new(current_fn),
                },
                current_tag,
            )),
        )
    } else {
        let (high, low) = split_cont_chain(tag, current_fn.built.continuation.take());
        current_fn.built.continuation = low;
        (
            high,
            Some((
                FunctionPtr {
                    function: Rc::new(current_fn),
                },
                current_tag,
            )),
        )
    }
}

fn setup_new_function(
    f: FunctionPtr,
    args: ResultVec<Value>,
    stack: &mut ResultVec<Value>,
    func_exec: &mut FuncExecData,
) -> VmResult<()> {
    if f.args_count != args.inner.len() as u32 {
        return Err(VmError::ArityMismatch {
            expected: f.args_count,
            actual: args.inner.len() as u32,
        });
    }

    assert!(f.function.is_built);

    let locals_count = f.locals_count;
    let upvars = f.built.upvars.clone();

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

fn continue_up(cp: ContPair) -> FunctionPtr {
    new_func(Function {
        name: Some("<continue-shim>".into()),
        built: BuiltFunction {
            upvars: vec![],
            continuation: Some(cp),
        },
        is_built: true,
        //upvars: vec![],
        instructions: vec![Instruction::Resume],
        args_count: 1,
        upvars_count: 0,
        locals_count: 0,
    })
}
