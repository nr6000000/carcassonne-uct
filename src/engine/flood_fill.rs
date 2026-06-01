use tinyvec::tiny_vec;

use crate::engine::datastructures::index::Index;

/// Span-based scanline flood fill.
///
/// Fills a connected region starting at `(x, y)` by processing entire
/// horizontal spans at once rather than individual pixels, making it
/// significantly more cache-friendly than naive BFS/DFS approaches.
///
/// # Arguments
/// * `index`  – Seed x and y index
/// * `inside` – Returns `true` for any pixel that should still be filled
/// * `set`    – Called exactly once per pixel to mark it as filled
pub fn flood_fill<F, S, R>(
    seed: Index,
    mut inside: F, 
    mut set: S,
    mut visited: R,
)
where
    F: FnMut(Index) -> bool,
    S: FnMut(Index),
    R: FnMut(Index) -> bool,
{
    // Ensure the starting pixel actually meets the condition
    if !inside(seed) {
        return;
    }
    
    // Mark the initial pixel immediately
    set(seed);
    
    // A 5x5 array has at most 25 pixels. A fixed stack of 32 entirely 
    // avoids heap allocations for small sizes, making it extremely fast.
    let mut stack = tiny_vec!([Index; 32]);
    stack.push(seed);
        
    while !stack.is_empty() {
        // Pop from the heap stack first if it's active, otherwise use the fast local stack
        let current = stack.pop().unwrap();
        
        let mut check_neighbor = |idx: Index| {
            if inside(idx) && !visited(idx) {
                set(idx); 
                stack.push(idx);
            }
        };

        // Check all 4 orthogonal directions
        check_neighbor(current + Index{x: 1, y: 0});
        check_neighbor(current + Index{x: 0, y: 1});
        check_neighbor(current + Index{x: -1, y: 0});
        check_neighbor(current + Index{x: 0, y: -1});
    }
}