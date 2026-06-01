use std::collections::HashSet;

use crate::game::datastructures::index::Index;

/// Tak naprawdę to nie:
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
pub fn flood_fill<F>(
    seed: Index,
    mut inside: F, 
) -> HashSet<Index>
where
    F: FnMut(Index) -> bool,
{
    // Ensure the starting pixel actually meets the condition
    if !inside(seed) {
        return HashSet::new();
    }

    let mut visited = HashSet::from([seed]);
    let mut stack = Vec::from([seed]);
        
    while !stack.is_empty() {
        // Pop from the heap stack first if it's active, otherwise use the fast local stack
        let current = stack.pop().unwrap();
        
        let mut check_neighbor = |idx: Index| {
            if inside(idx) && !visited.contains(&idx) {
                visited.insert(idx); 
                stack.push(idx);
            }
        };

        // Check all 4 orthogonal directions
        check_neighbor(current + Index{x: 1, y: 0});
        check_neighbor(current + Index{x: 0, y: 1});
        check_neighbor(current + Index{x: -1, y: 0});
        check_neighbor(current + Index{x: 0, y: -1});
    }

    visited
}