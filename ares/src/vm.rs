use linked_stack::{LinkedStack, LinkedStackBehavior};
use std::rc::Rc;
use continuation;
use function::FunctionPtr;
use value::Value;

#[derive(Clone, Eq, PartialEq, Debug)]
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
    Reset,
    Shift,
    Resume,
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
            Reset => {
                // The unique "identifier" for the reset
                let symbol = self.stack.pop()?.to_symbol()?;
                // The closure to execute underneath the reset
                let closure = self.stack.pop()?.to_function()?;
                // The reset closure must be argumentless
                assert!(closure.borrow().arg_count == 0);
                // Execute the reset closure using the symbol as the tag
                // for the new segment.  This closure exec is easier than
                // the above because it doesn't need to worry about argument
                // passing.
                self.stack.start_segment(
                    Some(symbol),
                    FuncExecData{function: closure, ip: 0});
            }
            Shift => {
                // The paired symbol for the shift. (this should be the
                // same as a symbol further up for a reset).
                let symbol = self.stack.pop()?.to_symbol()?;
                // The closure that is executed inside the shift
                let closure = self.stack.pop()?.to_function()?;
                // The shift closure takes exactly one argument (the closure)
                assert!(closure.borrow().arg_count == 1);

                // Split the stack on the symbol that we pulled out earlier.
                let cont_stack = self.stack.split(symbol)?;
                // Execute the shift closure with no tag
                self.stack.start_segment(None, FuncExecData { function: closure, ip: 0 });
                // The continuation is the argument to the closure.
                self.stack.push(Value::Continuation(Rc::new(continuation::Continuation {
                    stack: cont_stack
                })))?;
            }
            Resume  => {
                // The continuation is the first item
                let cont = self.stack.pop()?.to_continuation()?;
                // All continuations are resumed with a value.  This can be
                // null if a continuation doesn't expect a value
                let value = self.stack.pop()?;
                // Reconnect the continuation stack.
                self.stack.connect(cont.stack.clone());
                // The "value" of the continuation is the value that the
                // continuation was resumed with
                self.stack.push(value)?;
            }
        }

        Ok(StepResult::Continue)
    }
}
