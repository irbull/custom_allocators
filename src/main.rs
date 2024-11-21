#![feature(allocator_api)]
use custom_allocators::BumpAllocator;

fn main() {
    let bump_allocator = BumpAllocator::new();
    let mut my_vec: Vec<u8, &BumpAllocator> = Vec::with_capacity_in(1, &bump_allocator);
    for i in 0u32..128 {
        my_vec.push((i % 255).try_into().unwrap());
    }
    println!("{my_vec:?}"); // Outputs: [1, 2, 3, 4, 5]
}
