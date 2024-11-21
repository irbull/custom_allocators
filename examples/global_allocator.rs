use std::alloc::{GlobalAlloc, Layout};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::io::stdout;
use std::io::Write;
use std::sync::atomic::AtomicBool;
/// A simple bump allocator.
pub struct SimpleBumpAllocator;

const BUFFER_SIZE: usize = 1024 * 1024; // 1024 KB buffer

/// A statically aligned buffer. Ensures correct alignment of the base address.
#[repr(align(16))] // Align to 16 bytes, suitable for most types (e.g., u64)
struct AlignedBuffer([u8; BUFFER_SIZE]);

/// The static buffer, guaranteed to be aligned.
static mut BUFFER: AlignedBuffer = AlignedBuffer([0; BUFFER_SIZE]);

/// Atomic offset to track the current allocation position.
static OFFSET: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for SimpleBumpAllocator {
    #[allow(static_mut_refs)]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let current_offset = OFFSET.load(Ordering::Relaxed);

        // Align the offset to the layout's requirements
        let align_mask = layout.align() - 1;
        let aligned_offset = (current_offset + align_mask) & !align_mask;
        let new_offset = aligned_offset + layout.size();
        if new_offset > BUFFER_SIZE {
            std::ptr::null_mut() // Out of memory
        } else {
            //OFFSET.store(new_offset, Ordering::Relaxed);
            OFFSET.store(new_offset, Ordering::Relaxed);
            BUFFER.0.as_ptr().add(aligned_offset) as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // No-op: Deallocation is not supported in this bump allocator.
    }
}

/// Set the global allocator.
#[global_allocator]
static GLOBAL_ALLOCATOR: SimpleBumpAllocator = SimpleBumpAllocator;

fn main() {
    let v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]; // Allocates from the bump allocator
    println!("{:?}", v);
    let total_memory_allocated = OFFSET.load(Ordering::Relaxed);
    println!("Total memory allocated: {} bytes", total_memory_allocated);
}
