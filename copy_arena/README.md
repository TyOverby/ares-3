This Rust module provides a memory allocation Arena for types that
implement Copy.

[Documentation](https://PeterReid.github.io/copy_arena)

# Examples

Basic usage is as follows:

```rust
extern crate copy_arena;

use copy_arena::Arena;

let mut arena = Arena::new();
let mut allocator = arena.allocator();

let a: &mut i32 = allocator.alloc(5);
let b: &mut f64 = allocator.alloc_default();
let c: &mut [u8] = allocator.alloc_slice(b"some text");
let b: &mut [usize] = allocator.alloc_slice_fn(10, |idx| idx + 1);
let e: &mut [u32] = allocator.alloc_slice_default(10);
```

This could be used for building a tree structure without heap allocations for each node:

```rust
use copy_arena::{Arena, Allocator};

#[derive(Copy, Clone)]
struct Tree<'a> {
    left: Option<&'a Tree<'a>>,
    right: Option<&'a Tree<'a>>,
    content: u32
}

fn build_little_tree<'a>(allocator: &mut Allocator<'a>) -> &'a Tree<'a> {
    let left = allocator.alloc(Tree { left: None, right: None, content: 8 });
    let right = allocator.alloc(Tree { left: None, right: None, content: 16 });
    let parent = allocator.alloc(Tree { left: Some(left), right: Some(right), content: 13 });

    parent
}

#[test]
fn make_tree() {
    let mut arena = Arena::new();
    let mut allocator = arena.allocator();

    let root = build_little_tree(&mut allocator);
    assert_eq!(root.content, 13);
    assert_eq!(root.left.unwrap().content, 8);
    assert_eq!(root.right.unwrap().content, 16);
}
```

# Compared to std::arena::Arena

This differs from the (unstable) Arena in Rust's standard library in
a couple of ways:

 - This Arena only supports `Copy`-able objects -- no destructors are 
   executed.
 - This Arena does not use dynamic borrow checking, saving two RefCells
   and an Rc before getting to the underlying data to allocate from but
   leading to a slightly less convenient API.


