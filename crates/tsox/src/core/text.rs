pub type TextPos = i32;

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

    pub fn intersects(&self, other: &TextRange) -> bool {
        let start = self.pos.max(other.pos);
        let end = self.end.min(other.end);
        start <= end
    }
}

pub fn compare_text_ranges(r1: &TextRange, r2: &TextRange) -> std::cmp::Ordering {
    r1.pos.cmp(&r2.pos).then(r1.end.cmp(&r2.end))
}

#[cfg(test)]
mod tests;
