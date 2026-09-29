#![allow(unused_imports)]

use tsox_core::core::text::TextPos;

pub fn compute_line_of_position(line_starts: &[TextPos], pos: i32) -> usize {
    let mut low: isize = 0;
    let mut high: isize = line_starts.len() as isize - 1;
    while low <= high {
        let middle = low + ((high - low) >> 1);
        let value = line_starts[middle as usize];
        if value < pos {
            low = middle + 1;
        } else if value > pos {
            high = middle - 1;
        } else {
            return middle as usize;
        }
    }
    (low - 1) as usize
}
