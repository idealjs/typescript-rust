pub type TextPos = i32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct TextRange {
    pub pos: TextPos,
    pub end: TextPos,
}

impl TextRange {
    pub fn new(pos: usize, end: usize) -> Self { crate::fntrace::enter("new"); 
        Self {
            pos: pos as i32,
            end: end as i32,
        }
    }

    pub fn undefined() -> Self { crate::fntrace::enter("undefined"); 
        Self { pos: -1, end: -1 }
    }

    #[inline]
    pub fn pos(&self) -> usize { crate::fntrace::enter("pos"); 
        self.pos as usize
    }

    #[inline]
    pub fn end(&self) -> usize { crate::fntrace::enter("end"); 
        self.end as usize
    }

    #[inline]
    pub fn len(&self) -> usize { crate::fntrace::enter("len"); 
        (self.end - self.pos) as usize
    }

    #[inline]
    pub fn is_empty(&self) -> bool { crate::fntrace::enter("is_empty"); 
        self.pos == self.end
    }

    #[inline]
    pub fn is_valid(&self) -> bool { crate::fntrace::enter("is_valid"); 
        self.pos >= 0 || self.end >= 0
    }

    pub fn contains(&self, pos: usize) -> bool { crate::fntrace::enter("contains"); 
        (pos as i32) >= self.pos && (pos as i32) < self.end
    }

    pub fn contains_inclusive(&self, pos: usize) -> bool { crate::fntrace::enter("contains_inclusive"); 
        (pos as i32) >= self.pos && (pos as i32) <= self.end
    }

    pub fn contains_exclusive(&self, pos: usize) -> bool { crate::fntrace::enter("contains_exclusive"); 
        self.pos < (pos as i32) && (pos as i32) < self.end
    }

    pub fn with_pos(&self, pos: usize) -> Self { crate::fntrace::enter("with_pos"); 
        Self {
            pos: pos as i32,
            end: self.end,
        }
    }

    pub fn with_end(&self, end: usize) -> Self { crate::fntrace::enter("with_end"); 
        Self {
            pos: self.pos,
            end: end as i32,
        }
    }

    pub fn contained_by(&self, other: &TextRange) -> bool { crate::fntrace::enter("contained_by"); 
        other.pos <= self.pos && other.end >= self.end
    }

    pub fn overlaps(&self, other: &TextRange) -> bool { crate::fntrace::enter("overlaps"); 
        let start = self.pos.max(other.pos);
        let end = self.end.min(other.end);
        start < end
    }

    pub fn intersects(&self, other: &TextRange) -> bool { crate::fntrace::enter("intersects"); 
        let start = self.pos.max(other.pos);
        let end = self.end.min(other.end);
        start <= end
    }
}

pub fn compare_text_ranges(r1: &TextRange, r2: &TextRange) -> std::cmp::Ordering { crate::fntrace::enter("compare_text_ranges"); 
    r1.pos.cmp(&r2.pos).then(r1.end.cmp(&r2.end))
}
