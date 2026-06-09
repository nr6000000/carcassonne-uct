use rapidhash::RapidHashSet;

use crate::game_logic::datastructures::index::Index;

/// Tak naprawdę to nie:
/// Span-based scanline flood fill.
///
/// Fills a connected region starting at `(x, y)` by processing entire
/// horizontal spans at once rather than individual pixels, making it
/// significantly more cache-friendly than naive BFS/DFS approaches.
///
/// # Arguments
/// * `seed`  – Seed x and y index
/// * `inside` – Returns `true` for any pixel that should still be filled
pub fn flood_fill<F>(
    seed: Index,
    mut inside: F, 
) -> RapidHashSet<Index> 
where
    F: FnMut(Index) -> bool,
{
    let mut filled = RapidHashSet::default();
    
    // If the seed itself doesn't satisfy the predicate, return early
    if !inside(seed) {
        return filled;
    }

    // The stack stores horizontal segments to scan: (y, xl, xr, dy)
    // dy is the vertical direction (+1 for down, -1 for up) we arrived from.
    let mut stack = Vec::new();

    // 1. Find the initial horizontal span
    let mut xl = seed.x;
    while inside(Index{x: xl - 1, y: seed.y}) {
        xl -= 1;
    }

    let mut xr = seed.x;
    while inside(Index{x: xr + 1, y: seed.y}) {
        xr += 1;
    }

    // Mark the initial span as filled
    for x in xl..=xr {
        filled.insert(Index{x: x, y: seed.y});
    }

    // Push the lines immediately above and below the initial span
    stack.push((seed.y + 1, xl, xr, 1));
    stack.push((seed.y - 1, xl, xr, -1));

    // 2. Process segments from the stack
    while let Some((y, xl, xr, dy)) = stack.pop() {
        let mut x = xl;
        
        while x <= xr {
            // Check if the current pixel is unvisited and inside the region
            if !filled.contains(&Index{x, y}) && inside(Index{x, y}) {
                let mut l = x;
                
                // Leak to the left of the parent boundary
                while !filled.contains(&Index{x: l - 1, y}) && inside(Index{x: l - 1, y}) {
                    l -= 1;
                    filled.insert(Index{x: l, y});
                }
                
                // Heckbert Optimization: If we leaked left, check the opposite 
                // direction on the parent row to catch missed pockets.
                if l < xl {
                    stack.push((y - dy, l, xl - 1, -dy));
                }

                // Fill to the right within the parent boundary
                while x <= xr && !filled.contains(&Index{x, y}) && inside(Index{x, y}) {
                    filled.insert(Index{x, y});
                    x += 1;
                }
                let mut r = x - 1;

                // Leak to the right past the parent boundary
                while !filled.contains(&Index{x: r + 1, y}) && inside(Index{x: r + 1, y}) {
                    r += 1;
                    filled.insert(Index{x: r, y});
                }

                // Push the next row in the current direction
                stack.push((y + dy, l, r, dy));

                // Heckbert Optimization: If we leaked right, check the opposite
                // direction on the parent row to catch missed pockets.
                if r > xr {
                    stack.push((y - dy, xr + 1, r, -dy));
                }
                
                x = r + 1;
            } else {
                x += 1;
            }
        }
    }

    filled
        // Ensure the starting pixel actually meets the condition
    // if !inside(seed) {
    //     return HashSet::new();
    // }

    // let mut visited = HashSet::from([seed]);
    // let mut stack = Vec::from([seed]);
        
    // while !stack.is_empty() {
    //     // Pop from the heap stack first if it's active, otherwise use the fast local stack
    //     let current = stack.pop().unwrap();
        
    //     let mut check_neighbor = |idx: Index| {
    //         if inside(idx) && !visited.contains(&idx) {
    //             visited.insert(idx); 
    //             stack.push(idx);
    //         }
    //     };

    //     // Check all 4 orthogonal directions
    //     check_neighbor(current + Index{x: 1, y: 0});
    //     check_neighbor(current + Index{x: 0, y: 1});
    //     check_neighbor(current + Index{x: -1, y: 0});
    //     check_neighbor(current + Index{x: 0, y: -1});
    // }

    // visited
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_box_fill() {
        // 1 = Wall, 0 = Empty space
        let grid = [
            [1, 1, 1, 1, 1, 1, 1],
            [1, 0, 0, 0, 1, 1, 1],
            [1, 0, 0, 0, 1, 1, 1],
            [1, 0, 0, 0, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1],
        ];

        let seed = Index { x: 2, y: 2 };

        let result = flood_fill(
            seed,
            |idx| grid[idx.y as usize][idx.x as usize] == 0,
        );

        let expected = RapidHashSet::from_iter([
            Index {x: 1, y: 1},
            Index {x: 2, y: 1},
            Index {x: 3, y: 1},
            Index {x: 1, y: 2},
            Index {x: 2, y: 2},
            Index {x: 3, y: 2},
            Index {x: 1, y: 3},
            Index {x: 2, y: 3},
            Index {x: 3, y: 3},
        ]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_seed_on_boundary_does_nothing() {
        let grid = [
            [1, 1, 1],
            [1, 0, 1],
            [1, 1, 1],
        ]; // Simulating a smaller 3x3 subset for readability

        let seed = Index { x: 0, y: 0 }; // Starting directly on a wall

        let result = flood_fill(
            seed,
            |idx| grid[idx.y as usize][idx.x as usize] == 0,
        );

        assert!(result.is_empty());
    }

    #[test]
    fn test_u_shape_leak() {
        // A U-shaped cave where the algorithm has to flow down the left column,
        // loop around the bottom row, and fill up the right column.
        let grid = [
            [1, 1, 1, 1, 1, 1, 1],
            [1, 0, 1, 0, 0, 0, 1],
            [1, 0, 1, 0, 1, 0, 1],
            [1, 0, 1, 0, 1, 0, 1],
            [1, 0, 0, 0, 1, 0, 1],
            [1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1],
        ];

        let seed = Index { x: 1, y: 1 }; // Start at top left of the U

        let result = flood_fill(
            seed,
            |idx| grid[idx.y as usize][idx.x as usize] == 0,
        );

        let expected = RapidHashSet::from_iter([
            Index {x: 1, y: 1},
            Index {x: 1, y: 2},
            Index {x: 1, y: 3},
            Index {x: 1, y: 4},
            Index {x: 2, y: 4},
            Index {x: 3, y: 4},
            Index {x: 3, y: 3},
            Index {x: 3, y: 2},
            Index {x: 3, y: 1},
            Index {x: 4, y: 1},
            Index {x: 5, y: 1},
            Index {x: 5, y: 2},
            Index {x: 5, y: 3},
            Index {x: 5, y: 4},
        ]);

        assert_eq!(result, expected);

    }

    #[test]
    fn test_no_diagonal_bleeding() {
        let grid = [
            [1, 1, 1, 1, 1, 1, 1],
            [1, 0, 0, 1, 0, 0, 1],
            [1, 0, 0, 1, 0, 0, 1],
            [1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1],
        ];

        let seed = Index { x: 1, y: 1 }; // Start in the left room

        let result = flood_fill(
            seed,
            |idx| grid[idx.y as usize][idx.x as usize] == 0,
        );

        // Left room should be filled (2s), right room must remain un-filled (0s)
        assert!(result.len() == 4);
        assert!(result.contains(&Index{x: 1, y: 1}));
        assert!(result.contains(&Index{x: 2, y: 1}));
        assert!(result.contains(&Index{x: 1, y: 2}));
        assert!(result.contains(&Index{x: 2, y: 2}));
    }
}