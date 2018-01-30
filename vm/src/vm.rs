use linked_stack::{LinkedStack, LinkedStackBehavior};
use std::rc::Rc;
use continuation;
use function::FunctionPtr;
use value::{AresMap, AresObj, Value, ValueKind};
use std::fmt::{Debug, Formatter, Result as FmtResult};

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Symbol(pub &'static str);

pub type VmResult<T> = Result<T, VmError>;

#[derive(Clone, PartialEq, Debug)]
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
    KeyNotFound(Value),
    FieldNotFound(Symbol),
    ArityMismatch { actual: usize, expected: usize },
    TagNotFound(Symbol),
    UnexpectedType { expected: ValueKind, found: Value },
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
    Reset,
    Shift,
    Resume,

    MapEmpty,
    MapInsert,
    MapGet,

    ObjEmpty,
    ObjInsert,
    ObjGet,
}

#[derive(Clone, PartialEq, Debug)]
pub struct FuncExecData {
    function: FunctionPtr,
    ip: usize,
}

#[derive(PartialEq, Clone, Debug)]
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
        assert_eq!(function.borrow().arg_count, 0);

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
            Push(v) => {
                self.stack.push(v)?;
            }

            ObjEmpty => {
                self.stack.push(Value::Obj(AresObj::new()))?;
            }
            ObjInsert => {
                let obj = self.stack.pop()?.into_obj()?;
                let v = self.stack.pop()?;
                let k = self.stack.pop()?.into_symbol()?;
                let obj = obj.plus(k, v);
                self.stack.push(Value::Obj(obj))?;
            }
            ObjGet => {
                let k = self.stack.pop()?.into_symbol()?;
                let obj = self.stack.pop()?.into_obj()?;
                if let Some(v) = obj.find(&k) {
                    self.stack.push(v.clone())?;
                } else {
                    return Err(VmError::FieldNotFound(k));
                }
            }

            MapEmpty => {
                self.stack.push(Value::Map(AresMap::new()))?;
            }
            MapInsert => {
                let map = self.stack.pop()?.into_map()?;
                let v = self.stack.pop()?;
                let k = self.stack.pop()?;
                let map = map.plus(k, v);
                self.stack.push(Value::Map(map))?;
            }
            MapGet => {
                let k = self.stack.pop()?;
                let map = self.stack.pop()?.into_map()?;
                if let Some(v) = map.find(&k) {
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
            Call => {
                let arg_count = self.stack.pop()?.into_int()?;
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
                let cont = Rc::new(continuation::Continuation { stack: cont_stack });
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
