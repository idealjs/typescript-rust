mod casing_gen;
mod identifier_gen;

use casing_gen::{
    SPECIAL_CASING_MAPPINGS, UNICODE_CASED_RANGES, UNICODE_CASE_IGNORABLE_RANGES,
};
use identifier_gen::{UNICODE_ESNEXT_IDENTIFIER_PART, UNICODE_ESNEXT_IDENTIFIER_START};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialCasingCondition {
    FinalSigma,
}

pub struct SpecialCasingMapping {
    pub lower: &'static str,
    pub upper: &'static str,
    pub conditional_lower: &'static str,
    pub condition: Option<SpecialCasingCondition>,
}

pub struct RangeTable {
    pub r16: &'static [(u32, u32, u32)],
    pub r32: &'static [(u32, u32, u32)],
}

impl RangeTable {
    pub fn contains(&self, r: u32) -> bool { crate::fntrace::enter("contains"); 
        in_ranges(self.r16, r, 0xFFFF) || in_ranges(self.r32, r, 0x10FFFF)
    }
}

fn in_ranges(ranges: &[(u32, u32, u32)], r: u32, max: u32) -> bool { crate::fntrace::enter("in_ranges"); 
    if r > max {
        return false;
    }
    let mut lo = 0usize;
    let mut hi = ranges.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        let (range_lo, range_hi, stride) = ranges[mid];
        if r < range_lo {
            hi = mid;
        } else if r > range_hi {
            lo = mid + 1;
        } else {
            return stride == 1 || (r - range_lo) % stride == 0;
        }
    }
    false
}

pub fn special_casing_mappings(r: u32) -> Option<&'static SpecialCasingMapping> { crate::fntrace::enter("special_casing_mappings"); 
    SPECIAL_CASING_MAPPINGS
        .binary_search_by_key(&r, |&(cp, _)| cp)
        .ok()
        .map(|idx| &SPECIAL_CASING_MAPPINGS[idx].1)
}

pub fn is_unicode_cased(r: u32) -> bool { crate::fntrace::enter("is_unicode_cased"); 
    UNICODE_CASED_RANGES.contains(r)
}

pub fn is_unicode_case_ignorable(r: u32) -> bool { crate::fntrace::enter("is_unicode_case_ignorable"); 
    UNICODE_CASE_IGNORABLE_RANGES.contains(r)
}

pub fn unicode_esnext_identifier_start(ch: char) -> bool { crate::fntrace::enter("unicode_esnext_identifier_start"); 
    UNICODE_ESNEXT_IDENTIFIER_START.contains(ch as u32)
}

pub fn unicode_esnext_identifier_part(ch: char) -> bool { crate::fntrace::enter("unicode_esnext_identifier_part"); 
    UNICODE_ESNEXT_IDENTIFIER_PART.contains(ch as u32)
}
