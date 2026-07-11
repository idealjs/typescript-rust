//! Binary search helpers, ported from `internal/core/binarysearch.go`.

use std::cmp::Ordering;

/// Binary search that assumes at most one element matches the target.
///
/// The comparison function receives the index and the element, and returns:
/// - `Less` if the target is after the element (search right)
/// - `Greater` if the target is before the element (search left)
/// - `Equal` if this element matches
///
/// Returns `(index, found)`. If not found, `index` is the insertion point.
///
/// Mirrors `core.BinarySearchUniqueFunc` in Go.
pub fn binary_search_unique_func<T, F>(slice: &[T], mut cmp: F) -> (usize, bool)
where
    F: FnMut(usize, &T) -> Ordering,
{
    let n = slice.len();
    if n == 0 {
        return (0, false);
    }
    let mut low = 0usize;
    let mut high = n - 1;
    while low <= high {
        let middle = low + ((high - low) >> 1);
        match cmp(middle, &slice[middle]) {
            Ordering::Less => low = middle + 1,
            Ordering::Greater => {
                if middle == 0 {
                    break;
                }
                high = middle - 1;
            }
            Ordering::Equal => return (middle, true),
        }
    }
    (low, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_found() {
        let data = [1, 3, 5, 7, 9];
        let (i, found) = binary_search_unique_func(&data, |_, x| x.cmp(&5));
        assert!(found);
        assert_eq!(i, 2);
    }

    #[test]
    fn search_not_found() {
        let data = [1, 3, 5, 7, 9];
        let (i, found) = binary_search_unique_func(&data, |_, x| x.cmp(&4));
        assert!(!found);
        assert_eq!(i, 2); // insertion point
    }

    #[test]
    fn search_empty() {
        let data: [i32; 0] = [];
        let (i, found) = binary_search_unique_func(&data, |_, _| Ordering::Equal);
        assert!(!found);
        assert_eq!(i, 0);
    }
}
