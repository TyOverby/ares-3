use super::vm::{VmError, VmResult};

#[derive(Debug, PartialEq, Eq)]
pub struct ResultVec<T> {
    pub inner: Vec<T>,
}

impl<T> ResultVec<T> {
    pub fn new() -> ResultVec<T> {
        ResultVec { inner: vec![] }
    }

    pub fn new_with(v: Vec<T>) -> ResultVec<T> {
        ResultVec { inner: v}
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


#[test]
fn pop_n_test() {
    let mut v = ResultVec::new();
    v.push(1).unwrap();
    v.push(2).unwrap();
    v.push(3).unwrap();
    v.push(4).unwrap();

    let r = v.pop_n(1).unwrap();

    assert_eq!(r.inner, vec![4]);
    assert_eq!(v.inner, vec![1, 2, 3]);

    let mut v = ResultVec::new();
    v.push(1).unwrap();
    v.push(2).unwrap();
    v.push(3).unwrap();
    v.push(4).unwrap();

    let r = v.pop_n(2).unwrap();

    assert_eq!(r.inner, vec![3, 4]);
    assert_eq!(v.inner, vec![1, 2]);
}
