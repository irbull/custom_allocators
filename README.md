# Custom Allocators

My deep dive into Rust memory started with building a custom allocator. While you might not need them every day, custom allocators can be a real game-changer when you need to improve performance or handle specific memory limitations.

Let’s explore what custom allocators are, why you might need one, and how to build a simple bump allocator in Rust. We’ll also discuss how to use this allocator for specific parts of your application and provide an option for using it globally.

## What Are Custom Allocators?

In most Rust programs, memory allocation happens automatically using the global allocator. By default, Rust uses `std::alloc::System`, which calls the system’s allocator (e.g., `malloc` on Linux or `HeapAlloc` on Windows). This works fine for most cases, but sometimes it’s not the best choice.

Custom allocators let you replace the global allocator or create specialized allocators for specific situations. They let you control how memory is used, which can help reduce waste, improve performance, or make memory usage more predictable. This can be especially helpful in:

- **Real-time systems**: Where you need to minimize delays.
- **Embedded devices**: With limited memory.
- **Games or simulations**: Where performance is critical, and you can predict memory use.
## Building a Simple Bump Allocator for Local Use

One of the simplest custom allocators is a **bump allocator**. It works by pre-allocating a block of memory and giving out parts of it in a straight line. It doesn’t support freeing individual allocations; instead, all memory is released at once when you reset the allocator. This makes bump allocators extremely fast, but they only work well in certain situations.

Here’s how we can create one in Rust for local use—specifically for allocating memory within a particular scope or for a single data structure.

### 1. Define the Allocator

Note: The `Allocator` API is currently unstable, so you need to use the nightly version of Rust and enable the feature `allocator_api`.

First, add the following to your `Cargo.toml` to use the nightly version:

```toml
[dependencies]
#![feature(allocator_api)]
```

Then, define the allocator:

```rust
#![feature(allocator_api)]
use std::alloc::{AllocError, Allocator, Layout};
use std::ptr::NonNull;
use std::sync::Mutex;

pub struct BumpAllocator {
    memory: Mutex<BumpMemory>,
}

struct BumpMemory {
    buffer: [u8; 1024], // Pre-allocated memory buffer
    offset: usize,      // Current allocation offset
}

impl BumpAllocator {
    pub fn new() -> Self {
        Self {
            memory: Mutex::new(BumpMemory {
                buffer: [0; 1024],
                offset: 0,
            }),
        }
    }
}

unsafe impl Allocator for BumpAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let mut memory = self.memory.lock().unwrap();
        let start = memory.offset;
        let end = start + layout.size();

        if end > memory.buffer.len() {
            Err(AllocError)
        } else {
            memory.offset = end;
            println!("Allocated {} from {start} to {}", end-start, end-1);
            let slice = &mut memory.buffer[start..end];
            Ok(NonNull::from(slice))
        }
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {
        // No-op: deallocation is unsupported in a bump allocator.
    }
}
```

## Using the Allocator Locally

Now that we have a bump allocator, let’s see how we can use it locally for specific data structures instead of globally.

### Example: Scoped Allocation with a Vec

Here’s how you can use the bump allocator for a single `Vec`:

```rust
#![feature(allocator_api)]

use allocator::BumpAllocator;

fn main() {
    let bump_allocator = BumpAllocator::new();
    let mut my_vec: Vec<u8, &BumpAllocator> = Vec::with_capacity_in(1, &bump_allocator);
    for i in 0u32..128 {
        my_vec.push((i % 255).try_into().unwrap());
    }
    println!("{:?}", my_vec); // Outputs: [1, 2, 3, 4, 5]
}
```

To run, use: `cargo +nightly run`

### Advantages of Localized Allocator Usage

- **Isolation**: Parts of your program can use custom allocators without affecting the rest.
- **Safety**: The custom allocator’s memory is freed when its owning scope ends, avoiding memory leaks.
- **Flexibility**: You can mix global and local allocators in the same program, making memory usage more efficient for specific parts.

### Using the Allocator Globally

If you want to use the bump allocator for the entire program, you can declare it as the global allocator using the `#[global_allocator]` attribute:

```rust
#![feature(allocator_api)]

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator::new();

fn main() {
 let v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]; // Allocates from the bump allocator
    println!("{:?}", v);
    let total_memory_allocated = OFFSET.load(Ordering::Relaxed);
    println!("Total memory allocated: {} bytes", total_memory_allocated);
}
```

By declaring the allocator globally, all allocations in your program—such as `Vec`, `Box`, or `String`—will use the bump allocator. There are lots of challenges to consider when using a bump allocator globally, such as fragmentation and memory leaks. Also, you cannot allocate memory during `alloc`, which makes debugging harder.

### Using Bumpalo for Efficient Bump Allocations

Another option for using bump allocation in Rust is the **Bumpalo** crate. Bumpalo is a popular, well-tested library that provides an easy-to-use bump allocator for efficient memory management in certain situations.

#### Example: Using Bumpalo

To use Bumpalo, add it to your `Cargo.toml`:

```toml
[dependencies]
bumpalo = "3"
```

Here’s a simple example of using Bumpalo for scoped memory allocations:

```rust
use bumpalo::Bump;

fn main() {
    let bump = Bump::new();
    // Allocate a vector using the bump allocator
    let numbers = bump.alloc_slice_copy(&[1, 2, 3, 4, 5]);
    println!("{:?}", numbers); // Outputs: [1, 2, 3, 4, 5]
}
```

In this example:

- We create a new `Bump` allocator using `Bump::new()`.
- The `alloc_slice_copy` method allocates memory for a slice of integers using the bump allocator.
- Bumpalo ensures that all memory allocated with it is automatically freed when the `bump` allocator goes out of scope.
- [Bumpalo can also be configured to use the unstable `allocator_api`](https://docs.rs/bumpalo/latest/bumpalo/#nightly-rust-allocator_api-support)

### Advantages of Using Bumpalo

- **Ease of Use**: Bumpalo provides a straightforward API for creating and using bump allocators without requiring the nightly version of Rust.
- **Efficiency**: Bumpalo is highly optimized for speed, making it a great choice for performance-critical sections of code.
- **Scoped Memory Management**: Like our custom bump allocator, memory allocated with Bumpalo is freed when the allocator goes out of scope, reducing the risk of memory leaks.

### Key Takeaways

- Rust’s allocator API is flexible enough to let you use custom allocators for specific parts of your program, rather than using them everywhere.
- Localized bump allocators are great for temporary or short-lived allocations, like tasks or object pools.
- You can also use custom allocators globally if the entire program benefits from specialized memory management.
- **Bumpalo** provides an easy-to-use and efficient alternative for bump allocations, and it works with stable Rust.

So the next time you’re working on a performance-critical or memory-constrained part of your app, try using a localized bump allocator or Bumpalo. It’s a powerful tool that gives you control—without sacrificing Rust’s safety guarantees!
