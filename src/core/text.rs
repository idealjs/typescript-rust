//! Text position and range types ported from `internal/core/text.go`.

/// A position within a source file, as a byte offset into the file text.
///
/// Mirrors `core.TextPos` in Go (an `i32`).
pub type TextPos = i32;

/// A half-open range `[pos, end)` within a source file.
///
/// Mirrors `core.TextRange` in Go.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct TextRange {
    pub pos: TextPos,
    pub end: TextPos,
}

impl TextRange {
    pub fn new(pos: usize, end: usize) -> Self {
        Self {
            pos: pos as i32,
            end: end as i32,
        }
    }

    /// The "undefined" range, used when a range is not applicable.
    pub fn undefined() -> Self {
        Self { pos: -1, end: -1 }
    }

    #[inline]
    pub fn pos(&self) -> usize {
        self.pos as usize
    }

    #[inline]
    pub fn end(&self) -> usize {
        self.end as usize
    }

    #[inline]
    pub fn len(&self) -> usize {
        (self.end - self.pos) as usize
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.pos == self.end
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        self.pos >= 0 || self.end >= 0
    }

    pub fn contains(&self, pos: usize) -> bool {
        (pos as i32) >= self.pos && (pos as i32) < self.end
    }

    pub fn contains_inclusive(&self, pos: usize) -> bool {
        (pos as i32) >= self.pos && (pos as i32) <= self.end
    }

    pub fn contains_exclusive(&self, pos: usize) -> bool {
        self.pos < (pos as i32) && (pos as i32) < self.end
    }

    pub fn with_pos(&self, pos: usize) -> Self {
        Self {
            pos: pos as i32,
            end: self.end,
        }
    }

    pub fn with_end(&self, end: usize) -> Self {
        Self {
            pos: self.pos,
            end: end as i32,
        }
    }

    pub fn contained_by(&self, other: &TextRange) -> bool {
        other.pos <= self.pos && other.end >= self.end
    }

    pub fn overlaps(&self, other: &TextRange) -> bool {
        let start = self.pos.max(other.pos);
        let end = self.end.min(other.end);
        start < end
    }

    /// Like `overlaps`, but touching ranges are considered intersecting.
    /// For example, `[0, 5)` intersects `[5, 10)`.
    pub fn intersects(&self, other: &TextRange) -> bool {
        let start = self.pos.max(other.pos);
        let end = self.end.min(other.end);
        start <= end
    }
}

/// Compare two text ranges by position, then by end.
pub fn compare_text_ranges(r1: &TextRange, r2: &TextRange) -> std::cmp::Ordering {
    r1.pos
        .cmp(&r2.pos)
        .then(r1.end.cmp(&r2.end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_basics() {
        let r = TextRange::new(5, 10);
        assert_eq!(r.pos(), 5);
        assert_eq!(r.end(), 10);
        assert_eq!(r.len(), 5);
        assert!(r.contains(5));
        assert!(r.contains(9));
        assert!(!r.contains(10));
        assert!(r.contains_inclusive(10));
    }

    #[test]
    fn range_overlap() {
        let a = TextRange::new(0, 5);
        let b = TextRange::new(5, 10);
        assert!(!a.overlaps(&b));
        assert!(a.intersects(&b));
    }
}
