use super::vm::{VmError, VmResult};

#[derive(Debug)]
pub struct ResultVec<T> {
    pub inner: Vec<T>,
}

impl<T> ResultVec<T> {
    pub fn new() -> ResultVec<T> {
        ResultVec { inner: vec![] }
    }

    pub fn get(&self, idx: u32) -> VmResult<&T> {
        if idx as usize >= self.inner.len() {
            Err(VmError::StackOverflow)
        } else {
            Ok(&self.inner[idx as usize])
        }
    }

    pub fn set(&mut self, idx: u32, value: T) -> VmResult<()> {
        if idx as usize >= self.inner.len() {
            Err(VmError::StackOverflow)
        } else {
            self.inner[idx as usize] = value;
            Ok(())
        }
    }

    pub fn push(&mut self, value: T) -> VmResult<()> {
        self.inner.push(value);
        Ok(())
    }

    pub fn pop(&mut self) -> VmResult<T> {
        self.inner.pop().ok_or(VmError::StackUnderflow)
    }

    pub fn peek(&self) -> VmResult<&T> {
        self.inner.first().ok_or(VmError::StackUnderflow)
    }

    #[allow(unused)]
    pub fn pop_n(&mut self, n: u32) -> VmResult<ResultVec<T>> {
        panic!("make sure this works correctly!");
        if self.inner.len() as u32 >= n {
            let il = self.inner.len();
            Ok(ResultVec {
                inner: self.inner.split_off(il - n as usize),
            })
        } else {
            Err(VmError::StackUnderflow)
        }
    }
}
