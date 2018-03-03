extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::marker::PhantomData;
use std::mem::swap;
use std::fmt::{Debug, Formatter, Result as FmtResult};

pub trait LinkedStackBehavior {
    type Symbol;
    type Error;

    fn overflow() -> Self::Error;
    fn underflow() -> Self::Error;
    fn tag_not_found(symbol: Self::Symbol) -> Self::Error;
}

#[derive(Clone, Eq, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct LinkedStack<T, K, A, B: LinkedStackBehavior> {
    tag: Option<K>,
    aux: A,
    current: Vec<T>,
    previous: Option<Box<LinkedStack<T, K, A, B>>>,
    _phantom: PhantomData<B>,
}

impl<T, K, A, B: LinkedStackBehavior> Debug for LinkedStack<T, K, A, B>
where
    T: Debug,
    K: Debug,
    A: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        if let Some(prev) = self.previous.as_ref() {
            Debug::fmt(prev, f)?;
        }
        if let &Some(ref tag) = &self.tag {
            write!(f, "{:?} [", tag)?;
        } else {
            write!(f, "[")?;
        }
        if f.alternate() {
            writeln!(f, "")?;
        }
        for v in &self.current {
            if f.alternate() {
                writeln!(f, "{:?},", v)?;
            } else {
                write!(f, "{:?},", v)?;
            }
        }
        write!(f, "]")?;

        Ok(())
    }
}

impl<T, K, A, B: LinkedStackBehavior<Symbol = K>> LinkedStack<T, K, A, B>
where
    B: LinkedStackBehavior<Symbol = K>,
    T: Clone,
{
    pub fn dup_from_pos_in_stackframe(&mut self, pos: u32) -> Result<(), B::Error> {
        if (pos as usize) < self.current.len() {
            let v = self.current[pos as usize].clone();
            return self.push(v);
        } else {
            return Err(B::overflow());
        }
    }
}
impl<T, K, A, B: LinkedStackBehavior<Symbol = K>> LinkedStack<T, K, A, B> {
    pub fn new(aux: A) -> LinkedStack<T, K, A, B> {
        LinkedStack {
            tag: None,
            aux: aux,
            current: vec![],
            previous: None,
            _phantom: PhantomData,
        }
    }
    pub fn new_with_tag(tag: K, aux: A) -> LinkedStack<T, K, A, B> {
        LinkedStack {
            tag: Some(tag),
            aux: aux,
            current: vec![],
            previous: None,
            _phantom: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.current.len() + self.previous.as_ref().map(|n| n.len()).unwrap_or(0)
    }

    pub fn link_len(&self) -> usize {
        1 + self.previous.as_ref().map(|n| n.link_len()).unwrap_or(0)
    }

    pub fn push(&mut self, t: T) -> Result<(), B::Error> {
        self.current.push(t);
        Ok(())
    }

    pub fn aux(&self) -> &A {
        &self.aux
    }

    pub fn aux_mut(&mut self) -> &mut A {
        &mut self.aux
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        if self.current.len() != 0 {
            self.current.last_mut()
        } else if let Some(prev) = self.previous.as_mut() {
            prev.peek_mut()
        } else {
            None
        }
    }

    pub fn peek(&self) -> Result<&T, B::Error> {
        if let Some(r) = self.current.last() {
            return Ok(r);
        }
        return Err(B::underflow());
    }

    pub fn pop(&mut self) -> Result<T, B::Error> {
        if let Some(r) = self.current.pop() {
            return Ok(r);
        }

        Err(B::underflow())
    }

    // TODO: Optimization oporitunity
    pub fn pop_n(&mut self, n: usize) -> Result<Vec<T>, B::Error> {
        let mut v = Vec::with_capacity(n);

        for _ in 0..n {
            v.push(self.pop()?)
        }

        Ok(v)
    }

    pub fn start_segment(&mut self, tag: Option<K>, aux: A) {
        let mut new = LinkedStack {
            tag: tag,
            aux: aux,
            current: vec![],
            previous: None,
            _phantom: PhantomData,
        };
        swap(self, &mut new);
        self.previous = Some(Box::new(new));
    }

    pub fn kill_segment(&mut self) -> Result<(), B::Error> {
        if let Some(prev) = self.previous.take() {
            *self = *prev;
            Ok(())
        } else {
            Err(B::underflow())
        }
    }

    pub fn split(&mut self, tag: K) -> Result<LinkedStack<T, K, A, B>, B::Error>
    where
        K: Eq,
    {
        fn split_impl<T, K, A, B: LinkedStackBehavior>(
            location: &mut LinkedStack<T, K, A, B>,
            tag: K,
        ) -> Result<Box<LinkedStack<T, K, A, B>>, K>
        where
            K: Eq,
        {
            if location.tag.as_ref() == Some(&tag) {
                match location.previous.take() {
                    Some(loc) => Ok(loc),
                    None => Err(tag),
                }
            } else if location.previous.is_some() {
                split_impl(location.previous.as_mut().unwrap(), tag)
            } else {
                Err(tag)
            }
        }

        let result = split_impl(self, tag)
            .map(|a| *a)
            .or_else(|tag| Err(B::tag_not_found(tag)));
        if let Ok(mut res) = result {
            swap(self, &mut res);
            return Ok(res);
        } else {
            return result;
        }
    }

    pub fn connect(&mut self, mut additional: LinkedStack<T, K, A, B>) {
        swap(self, &mut additional);
        let original = additional;

        fn connect_impl<T, K, A, B: LinkedStackBehavior>(
            target: &mut LinkedStack<T, K, A, B>,
            original: Box<LinkedStack<T, K, A, B>>,
        ) {
            if target.previous.is_none() {
                target.previous = Some(original);
            } else {
                connect_impl(target.previous.as_mut().unwrap(), original);
            }
        }

        connect_impl(self, Box::new(original))
    }

    pub fn iter(&self) -> Vec<&T> {
        let mut out = vec![];
        fn fill<'a, 'b, T, K, A, B: LinkedStackBehavior>(
            ls: &'a LinkedStack<T, K, A, B>,
            f: &'b mut Vec<&'a T>,
        ) {
            if let Some(next) = ls.previous.as_ref() {
                fill(next, f);
            }
            for v in &ls.current {
                f.push(v);
            }
        }

        fill(self, &mut out);
        out
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct TestStackBehavior;

    #[derive(Debug, Eq, PartialEq)]
    enum TestStackError {
        Underflow,
        Overflow,
        TagNotFound,
    }


    impl LinkedStackBehavior for TestStackBehavior {
        type Symbol = &'static str;
        type Error = TestStackError;

        fn underflow() -> Self::Error {
            TestStackError::Underflow
        }
        fn overflow() -> Self::Error {
            TestStackError::Overflow
        }
        fn tag_not_found(_symbol: Self::Symbol) -> Self::Error {
            TestStackError::TagNotFound
        }
    }

    type TestLinkedStack = LinkedStack<u32, &'static str, &'static str, TestStackBehavior>;

    #[test]
    fn basic_push_pop() {
        let mut stack: TestLinkedStack = LinkedStack::new("hello");
        assert_eq!(*stack.aux(), "hello");

        assert!(stack.push(1).is_ok());
        assert_eq!(stack.pop(), Ok(1));
    }

    #[test]
    fn pop_off_end() {
        let mut stack: TestLinkedStack = LinkedStack::new("hello");
        assert_eq!(stack.pop(), Err(TestStackError::Underflow));
    }

    #[test]
    fn pop_between_funcs() {
        let mut stack: TestLinkedStack = LinkedStack::new("hello");
        assert!(stack.push(1).is_ok());
        stack.start_segment(None, "bye");
        assert_eq!(stack.pop(), Err(TestStackError::Underflow));
        assert_eq!(stack.peek(), Err(TestStackError::Underflow));
    }

    #[test]
    fn kill_segment() {
        let mut stack: TestLinkedStack = LinkedStack::new("hello");
        assert_eq!(stack.link_len(), 1);
        stack.start_segment(None, "hi");
        assert_eq!(stack.link_len(), 2);
        stack.kill_segment();
        assert_eq!(stack.link_len(), 1);
        assert_eq!(*stack.aux(), "hello");
    }

    #[test]
    fn connect_segment() {
        let mut stack1: TestLinkedStack = LinkedStack::new("hi");
        let mut stack2: TestLinkedStack = LinkedStack::new("bye");

        stack1.push(1);
        stack2.push(2);

        stack1.connect(stack2);

        assert_eq!(stack1.pop(), Ok(2));
    }

    #[test]
    fn test_split() {
        let mut stack: TestLinkedStack = LinkedStack::new("hi");
        stack.start_segment(Some("x"), "");
        stack.start_segment(Some("y"), "");
        stack.start_segment(Some("z"), "");

        let after = stack.split(&"y").unwrap();
        assert_eq!(stack.link_len(), 2);
        assert_eq!(after.link_len(), 2);

        assert_eq!(stack.tag, Some("x"));
    }
}
