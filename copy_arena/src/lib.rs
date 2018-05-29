//! A memory allocation arena specialized for types implement Copy.
//!
//! An `Arena` can allocate objects more efficiently than a
//! general-purpose allocator, but the objects cannot be
//! deallocated until the `Arena` itself is destroyed.
//!
//! To actually allocate out of an `Arena` after creating it,
//! use its `allocator()` method to create an `Allocator`.
//!
//! # Examples
//!
//! Basic usage works as follows:
//!
//! ```
//! use copy_arena::Arena;
//!
//! let mut arena = Arena::new();
//! let mut allocator = arena.allocator();
//!
//! let a: &mut i32 = allocator.alloc(5);
//! let b: &mut f64 = allocator.alloc_default();
//! let c: &mut [u8] = allocator.alloc_slice(b"some text");
//! let b: &mut [usize] = allocator.alloc_slice_fn(10, |idx| idx + 1);
//! let e: &mut [u32] = allocator.alloc_slice_default(10);
//! ```
//!
//! A slightly more realistic use case would be to create a tree without
//! allocating from the heap for each node.
//!
//! ```
//! use copy_arena::{Arena, Allocator};
//!
//! #[derive(Copy, Clone)]
//! struct Tree<'a> {
//!     left: Option<&'a Tree<'a>>,
//!     right: Option<&'a Tree<'a>>,
//!     content: u32
//! }
//!
//! fn build_little_tree<'a>(allocator: &mut Allocator<'a>) -> &'a Tree<'a> {
//!     let left = allocator.alloc(Tree { left: None, right: None, content: 8 });
//!     let right = allocator.alloc(Tree { left: None, right: None, content: 16 });
//!     let parent = allocator.alloc(Tree { left: Some(left), right: Some(right), content: 13 });
//!
//!     parent
//! }
//!
//!
//! let mut arena = Arena::new();
//! let mut allocator = arena.allocator();
//!
//! let root = build_little_tree(&mut allocator);
//! assert_eq!(root.content, 13);
//! assert_eq!(root.left.unwrap().content, 8);
//! assert_eq!(root.right.unwrap().content, 16);
//!
//! ```

use std::cmp;
use std::default::Default;
use std::fmt;
use std::mem;
use std::slice;
use std::usize;

struct Chunk {
    data: Vec<u8>,
    next: Option<Box<Chunk>>,
}

impl Chunk {
    fn attempt_alloc(&mut self, size: usize, align: usize) -> Option<*mut u8> {
        let start = round_up(self.data.len(), align);

        if size <= self.data.capacity() && start <= self.data.capacity() - size {
            Some(unsafe {
                self.data.set_len(start + size);
                self.data.as_mut_ptr().offset(start as isize)
            })
        } else {
            None
        }
    }
}

/// Holds the backing memory for allocated objects out of itself.
///
/// Actual allocation into the `Arena` happens via an `Allocator` returned
/// from `allocator()`.
pub struct Arena {
    head: Chunk,
}

impl Arena {
    /// Construct a new Arena. This _does_ allocate an block of memory.
    pub fn new() -> Arena {
        Arena::with_capacity(1000)
    }

    /// Construct a new Arena with the given initial capacity.
    ///
    /// The chosen capacity does not limit the final size of the arena.
    pub fn with_capacity(capacity: usize) -> Arena {
        Arena {
            head: Chunk {
                data: Vec::with_capacity(capacity),
                next: None,
            },
        }
    }

    fn add_chunk(&mut self, chunk_size: usize) {
        let mut new_head = Chunk {
            data: Vec::with_capacity(chunk_size),
            next: None,
        };

        mem::swap(&mut self.head, &mut new_head);
        self.head.next = Some(Box::new(new_head));
    }

    /// Construct an Allocator for this arena.
    pub fn allocator(&mut self) -> Allocator {
        Allocator { arena: self }
    }

    /// Get the number of bytes of memory that have been allocated
    /// in service of this arena. Not all of this capacity is necessarily
    /// useful, since asked-for memory may not perfectly fit in the
    /// underlying blocks allocated.
    pub fn capacity(&self) -> usize {
        let mut iter: &Chunk = &self.head;
        let mut total_capacity = 0;
        loop {
            total_capacity += iter.data.capacity();
            match iter.next {
                None => {
                    return total_capacity;
                }
                Some(ref next) => {
                    iter = next;
                }
            }
        }
    }
}

impl fmt::Debug for Arena {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "Arena {{ capacity_bytes: {} }}",
            self.capacity()
        ))
    }
}

/// Allows allocation out of arena.
#[derive(Debug)]
pub struct Allocator<'a> {
    arena: &'a mut Arena,
}

#[inline]
fn round_up(base: usize, align: usize) -> usize {
    (base + (align - 1)) & !(align - 1)
}

impl<'a> Allocator<'a> {
    fn alloc_raw(&mut self, size: usize, align: usize) -> &'a mut u8 {
        loop {
            match self.arena.head.attempt_alloc(size, align) {
                Some(x) => return unsafe { mem::transmute(x) },
                None => {
                    // Double the current allocation (or the asked for one), but don't overflow.
                    let minimum_reasonable = cmp::max(self.arena.head.data.len(), size);
                    let new_chunk_size = 2 * cmp::min(minimum_reasonable, usize::MAX / 2);
                    self.arena.add_chunk(new_chunk_size);
                }
            }
        }
    }

    /// Allocate a copy of an object
    pub fn alloc<T: Copy>(&mut self, elem: T) -> &'a mut T {
        let memory = self.alloc_raw(mem::size_of::<T>(), mem::align_of::<T>());
        let res: &'a mut T = unsafe { mem::transmute(memory) };
        *res = elem;
        res
    }

    /// Allocate a default-valued object
    pub fn alloc_default<T: Copy + Default>(&mut self) -> &'a mut T {
        self.alloc(Default::default())
    }

    /// Allocate and leave uninitialized a slice of the given length
    fn alloc_slice_raw<T>(&mut self, len: usize) -> &'a mut [T] {
        let element_size = mem::size_of::<[T; 2]>() / 2;
        debug_assert_eq!(mem::size_of::<[T; 7]>(), 7 * element_size);
        let byte_count = element_size
            .checked_mul(len)
            .expect("Arena slice size overflow");
        let memory = self.alloc_raw(byte_count, mem::align_of::<T>());
        let res: &'a mut [T] = unsafe { slice::from_raw_parts_mut(mem::transmute(memory), len) };
        res
    }

    /// Allocate a copy of a slice
    pub fn alloc_slice<T: Copy>(&mut self, elems: &[T]) -> &'a mut [T] {
        let slice = self.alloc_slice_raw(elems.len());
        for (dest, src) in slice.iter_mut().zip(elems.iter()) {
            *dest = *src;
        }
        slice
    }

    // Allocate a string in the Arena
    pub fn alloc_str<S: AsRef<str>>(&mut self, s: S) -> &'a str {
        let bytes = s.as_ref().as_bytes();
        let alloced = self.alloc_slice(bytes);
        if cfg!(debug) {
            std::str::from_utf8(alloced).unwrap()
        } else {
            unsafe { std::str::from_utf8_unchecked(alloced) }
        }
    }

    // Allocates an iterator to the arena.
    // If the iterator doesnt have a max size, it is first collected into a vec
    // and then allocated.
    pub fn alloc_iter<I, T>(&mut self, iterator: I) -> &'a mut [T]
    where
        T: Copy,
        I: IntoIterator<Item = T>,
    {
        let mut iterator = iterator.into_iter();
        if let (_, Some(max)) = iterator.size_hint() {
            self.alloc_slice_opt_fn(max, |_| iterator.next())
        } else {
            let collected: Vec<T> = iterator.collect();
            self.alloc_slice(&collected)
        }
    }

    /// Allocate and populate a slice, creating each element as a function
    /// of its index.
    pub fn alloc_slice_opt_fn<T: Copy, F>(&mut self, len: usize, mut f: F) -> &'a mut [T]
    where
        F: FnMut(usize) -> Option<T>,
    {
        let slice = self.alloc_slice_raw(len);
        let mut i = 0;
        for idx in 0..len {
            if let Some(r) = f(idx) {
                slice[i] = r;
                i += 1;
            }
        }
        &mut slice[..i]
    }

    /// Allocate and populate a slice, creating each element as a function
    /// of its index.
    pub fn alloc_slice_fn<T: Copy, F>(&mut self, len: usize, mut f: F) -> &'a mut [T]
    where
        F: FnMut(usize) -> T,
    {
        self.alloc_slice_opt_fn(len, |i| Some(f(i)))
    }

    /// Allocate a slice populated by default-valued elements.
    pub fn alloc_slice_default<T: Copy + Default>(&mut self, len: usize) -> &'a mut [T] {
        self.alloc_slice_fn(len, |_| Default::default())
    }
}

#[test]
fn construct_simple() {
    let mut arena = Arena::with_capacity(4);
    let mut allocator = arena.allocator();

    let x: &mut i32 = allocator.alloc(44);
    let y: &mut u8 = allocator.alloc(3);
    let z: &mut u32 = allocator.alloc(0x11223344);
    let w: &mut f64 = allocator.alloc_default();
    assert_eq!(*x, 44);
    assert_eq!(*y, 3);
    assert_eq!(*z, 0x11223344);
    assert_eq!(*w, 0.0);
}

#[test]
fn just_enough_capacity() {
    let mut arena = Arena::with_capacity(7);
    arena.allocator().alloc_slice_default::<u8>(7);

    assert!(arena.head.next.is_none());
    assert_eq!(arena.head.data.len(), 7);
}

#[test]
fn barely_too_little_capacity() {
    let mut arena = Arena::with_capacity(7);
    arena.allocator().alloc_slice_default::<u8>(8);

    assert!(arena.head.next.is_some());
    assert_eq!(arena.head.data.len(), 8);
}

#[test]
fn many_u8s() {
    let mut arena = Arena::with_capacity(52);
    for i in 0..100 {
        assert_eq!(*arena.allocator().alloc(4), 4u8);
        assert_eq!(arena.head.data.len(), (i % 52) + 1);
    }
    assert!(arena.head.data.capacity() > 52);
}

#[test]
fn zero_capacity() {
    let mut arena = Arena::with_capacity(0);
    assert_eq!(arena.head.data.capacity(), 0);

    assert_eq!(*arena.allocator().alloc(9u32), 9u32);

    assert!(arena.head.data.capacity() >= 4);
    assert!(arena.head.data.len() == 4);
}

#[test]
fn surprisingly_large() {
    // Make sure asking for an allocation that is larger than the
    // block we otherwise would have allocated works fine.
    let mut arena = Arena::with_capacity(10);

    {
        let mut allocator = arena.allocator();
        let x = allocator.alloc(7u32);
        let ys: &[u8] = allocator.alloc_slice_default(600);

        assert_eq!(*x, 7u32);
        assert_eq!(ys.len(), 600);
    }

    assert!(arena.head.next.is_some());
    assert!(arena.head.data.capacity() >= 600);
    assert_eq!(arena.head.data.len(), 600);
}

#[test]
fn construct_slices() {
    let mut arena = Arena::with_capacity(4);
    let mut allocator = arena.allocator();

    let s = ::std::str::from_utf8(allocator.alloc_slice(b"abc")).unwrap();
    let xs: &[i32] = allocator.alloc_slice_fn(10, |idx| (idx as i32) * 7);
    let ys: &[u64] = allocator.alloc_slice_default(4);

    assert_eq!(xs[9], 9 * 7);
    assert_eq!(s, "abc");
    assert_eq!(ys[0], 0);
}

#[test]
fn zero_size() {
    let mut arena = Arena::with_capacity(4);
    {
        let mut allocator = arena.allocator();

        let many_units = allocator.alloc_slice_fn(500, |_| ());
        assert_eq!(many_units.len(), 500);
    }
    assert_eq!(arena.capacity(), 4);

    {
        let mut allocator = arena.allocator();

        let many_units: Vec<&mut ()> = (0..500).map(|_| allocator.alloc(())).collect();
        assert_eq!(many_units.len(), 500);
    }
    assert_eq!(arena.capacity(), 4);
}
