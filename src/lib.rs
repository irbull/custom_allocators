#![feature(allocator_api)]
#![feature(const_trait_impl)]
use std::alloc::GlobalAlloc;
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

impl Default for BumpAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl BumpAllocator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            memory: Mutex::new(BumpMemory {
                buffer: [0; 1024],
                offset: 0,
            }),
        }
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut memory = self.memory.lock().unwrap();

        let start = memory.offset;
        let end = start + layout.size();

        if end > memory.buffer.len() {
            std::ptr::null_mut() // Out of memory
        } else {
            memory.offset = end;
            println!("Allocated {} from {start} to {}", end - start, end - 1);

            memory.buffer.as_mut_ptr().add(start)
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // No-op: dealloc is unsupported in a bump allocator.
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
            println!("Allocated {} from {start} to {}", end - start, end - 1);
            let slice = &mut memory.buffer[start..end];
            Ok(NonNull::from(slice))
        }
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {
        // No-op: deallocation is unsupported in a bump allocator.
    }
}
