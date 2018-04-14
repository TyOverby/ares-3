use value::{new_func, AresMap, Continuation, ContinuationPtr, FunctionPtr, Symbol, Value,
            ValueKind};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter, Result as FmtResult};
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
    Call(u32),
    TailCall(u32),
    Reset,
    Shift,
    Resume,
    Terminate,

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
        function_stack: &mut ResultVec<Value>,
    ) -> VmResult<StepResult> {
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

        self.apply_instr(instruction, function_stack)
    }

    fn apply_instr(
        &mut self,
        instruction: Instruction,
        stack: &mut ResultVec<Value>,
    ) -> VmResult<StepResult> {
        fn assert_numeric(v: &Value) -> VmResult<()> {
            if v.kind() != ValueKind::Integer && v.kind() != ValueKind::Float {
                return Err(VmError::UnexpectedType {
                    expected: ValueKind::Integer,
                    found: v.clone(),
                });
            }
            return Ok(());
        }

        use self::Instruction::*;
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
                let f = stack.pop()?.into_function()?;
                let mut function = f.borrow().clone();
                assert!(function.upvars.len() == 0);
                let upvars = stack.pop_n(function.upvars_count)?;
                function.upvars = upvars.inner;
                stack.push(Value::Function(new_func(function)))?;
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
            TailCall(arg_count) => {
                let args = stack.pop_n(arg_count)?;
                let f = stack.pop()?.into_function()?;

                if f.borrow().args_count != arg_count {
                    return Err(VmError::ArityMismatch {
                        expected: f.borrow().args_count,
                        actual: arg_count,
                    });
                }

                let locals_count = f.borrow().locals_count;
                let upvars = f.borrow().upvars.clone();

                let exec_data = FuncExecData {
                    function: f.clone(),
                    ip: 0,
                };

                //stack.start_segment(None, exec_data);

                //stack.push(Value::Function(f))?;

                for arg in args.inner {
                    stack.push(arg)?;
                }

                for upvar in upvars {
                    stack.push(upvar.clone())?;
                }

                for _ in 0..locals_count {
                    stack.push(Value::Integer(9999999999))?;
                }
            }
            Call(_arg_count) => {
                unimplemented!();
            }
            Terminate => {
                let result = stack.pop()?;
                return Ok(StepResult::Done(result));
            }
            Reset => {
                unimplemented!();
            }
            Shift => {
                unimplemented!();
            }
            Resume => {
                unimplemented!();
            }
        }

        Ok(StepResult::Continue)
    }
}
