use linked_stack::{LinkedStack, LinkedStackBehavior};
use value::{AresMap, Continuation, ContinuationPtr, FunctionPtr, Value, ValueKind};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter, Result as FmtResult};

#[derive(Clone, Eq, PartialEq, Hash, PartialOrd, Serialize, Deserialize)]
pub struct Symbol(pub String);

pub type VmResult<T> = Result<T, VmError>;

#[derive(Clone, PartialEq, Debug, PartialOrd, Serialize, Deserialize)]
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

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum VmError {
    StackUnderflow,
    StackOverflow,
    CrossBoundary,
    KeyNotFound(Value),
    FieldNotFound(Symbol),
    ArityMismatch { actual: usize, expected: usize },
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

    Call(u32),
    Ret,
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
    pub(crate) stack: ValueStack,
    pub(crate) debug_values: Vec<Value>,
    pub(crate) modules: HashMap<(Symbol, Symbol), Value>,
}

#[derive(PartialEq, Clone, Debug)]
pub enum StepResult {
    Done(Value),
    Continue,
}

impl Vm {
    pub fn new(function: FunctionPtr) -> Vm {
        assert_eq!(function.borrow().arg_count, 0);

        let exec_data = FuncExecData {
            function: function,
            ip: 0,
        };

        Vm {
            stack: ValueStack::new(exec_data),
            debug_values: vec![],
            modules: HashMap::new(),
        }
    }

    pub fn run(&mut self) -> VmResult<Value> {
        loop {
            if let StepResult::Done(v) = self.step()? {
                return Ok(v);
            }
        }
    }

    pub fn step(&mut self) -> VmResult<StepResult> {
        let instruction = {
            let &FuncExecData {
                ref function,
                ref ip,
            } = self.stack.aux();
            if *ip >= function.borrow().instructions.len() {
                return Err(VmError::RanOutOfInstructions);
            }
            function.borrow().instructions[*ip].clone()
        };

        self.stack.aux_mut().ip += 1;

        self.apply_instr(instruction)
    }

    fn apply_instr(&mut self, instruction: Instruction) -> VmResult<StepResult> {
        use self::Instruction::*;
        match instruction {
            Add => {
                let l = self.stack.pop()?.into_int()?;
                let r = self.stack.pop()?.into_int()?;
                self.stack.push(Value::Integer(l + r))?;
            }
            Sub => {
                let l = self.stack.pop()?.into_int()?;
                let r = self.stack.pop()?.into_int()?;
                self.stack.push(Value::Integer(l - r))?;
            }
            Mul => {
                let l = self.stack.pop()?.into_int()?;
                let r = self.stack.pop()?.into_int()?;
                self.stack.push(Value::Integer(l * r))?;
            }
            Div => {
                let l = self.stack.pop()?.into_int()?;
                let r = self.stack.pop()?.into_int()?;
                self.stack.push(Value::Integer(l / r))?;
            }

            GetFromStackPosition(pos) => {
                self.stack.dup_from_pos_in_stackframe(pos)?;
            }
            SetToStackPosition(pos) => {
                self.stack.dup_from_pos_in_stackframe(pos)?;
            }
            Push(v) => {
                self.stack.push(v)?;
            }
            Pop => {
                self.stack.pop()?;
            }

            ModuleAdd => {
                let module_name = self.stack.pop()?.into_symbol()?;
                let definition_name = self.stack.pop()?.into_symbol()?;
                let value = self.stack.pop()?;
                self.modules.insert((module_name, definition_name), value);
            }
            ModuleGet => {
                let module_name = self.stack.pop()?.into_symbol()?;
                let definition_name = self.stack.pop()?.into_symbol()?;
                let value = self.modules
                    .get(&(module_name.clone(), definition_name.clone()));

                let value = value.ok_or_else(|| {
                    VmError::NoModuleDefinition {
                        module: module_name,
                        definition: definition_name,
                    }
                })?;

                self.stack.push(value.clone())?;
            }

            MapEmpty => {
                self.stack.push(Value::Map(AresMap::new()))?;
            }
            MapInsert => {
                let map = self.stack.pop()?.into_map()?;
                let v = self.stack.pop()?;
                let k = self.stack.pop()?;
                let map = map.insert(k, v);
                self.stack.push(Value::Map(map))?;
            }
            MapGet => {
                let k = self.stack.pop()?;
                let map = self.stack.pop()?.into_map()?;
                if let Some(v) = map.get(&k) {
                    self.stack.push(v.clone())?;
                } else {
                    return Err(VmError::KeyNotFound(k));
                }
            }
            Dup => {
                let v = self.stack.peek()?.clone();
                self.stack.push(v)?;
            }
            Print => {
                println!("{:?}", self.stack);
            }
            Call(arg_count) => {
                let f = self.stack.pop()?.into_function()?;

                if f.borrow().arg_count != arg_count as usize {
                    return Err(VmError::ArityMismatch {
                        expected: f.borrow().arg_count,
                        actual: arg_count as usize,
                    });
                }

                let args = self.stack.pop_n(arg_count as usize)?;
                let exec_data = FuncExecData { function: f, ip: 0 };
                self.stack.start_segment(None, exec_data);
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
            Reset => {
                let symbol = self.stack.pop()?.into_symbol()?;
                let closure = self.stack.pop()?.into_function()?;
                assert_eq!(closure.borrow().arg_count, 0);

                let exec_data = FuncExecData {
                    function: closure,
                    ip: 0,
                };
                self.stack.start_segment(Some(symbol), exec_data);
            }
            Shift => {
                let symbol = self.stack.pop()?.into_symbol()?;
                let closure = self.stack.pop()?.into_function()?;
                assert_eq!(closure.borrow().arg_count, 1);

                let cont_stack = self.stack.split(symbol)?;
                self.stack.start_segment(
                    None,
                    FuncExecData {
                        function: closure,
                        ip: 0,
                    },
                );
                let cont = ContinuationPtr::new(Continuation { stack: cont_stack });
                self.stack.push(Value::Continuation(cont))?;
            }
            Resume => {
                let value = self.stack.pop()?;
                let cont = self.stack.pop()?.into_continuation()?;

                self.stack.connect(cont.stack.clone());
                self.stack.push(value)?;
            }
        }

        Ok(StepResult::Continue)
    }
}

impl Debug for Symbol {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "'{}", self.0)
    }
}
