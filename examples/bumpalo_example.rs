use bumpalo::Bump;

fn main() {
    let bump = Bump::new();
    // Allocate a vector using the bump allocator
    let numbers = bump.alloc_slice_copy(&[1, 2, 3, 4, 5]);
    println!("{:?}", numbers); // Outputs: [1, 2, 3, 4, 5]
}
