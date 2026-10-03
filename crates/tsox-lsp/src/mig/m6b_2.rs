use std::cmp::Ordering;
use std::sync::OnceLock;

use tsox_core::core::text::TextRange;

pub type TextPos = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Verbatim,
    Atom,
    Alias,
}

pub type Feature = i32;

pub const FEATURE_HOVER: Feature = 1;
pub const FEATURE_SIGNATURE_HELP: Feature = 1 << 1;
pub const FEATURE_COMPLETION: Feature = 1 << 2;
pub const FEATURE_DEFINITION: Feature = 1 << 3;
pub const FEATURE_TYPE_DEFINITION: Feature = 1 << 4;
pub const FEATURE_IMPLEMENTATION: Feature = 1 << 5;
pub const FEATURE_REFERENCES: Feature = 1 << 6;
pub const FEATURE_DOCUMENT_HIGHLIGHTS: Feature = 1 << 7;
pub const FEATURE_RENAME: Feature = 1 << 8;
pub const FEATURE_CALL_HIERARCHY: Feature = 1 << 9;
pub const FEATURE_CODE_ACTIONS: Feature = 1 << 10;
pub const FEATURE_FORMATTING: Feature = 1 << 11;
pub const FEATURE_INLAY_HINTS: Feature = 1 << 12;
pub const FEATURE_SEMANTIC_TOKENS: Feature = 1 << 13;
pub const FEATURE_FOLDING_RANGES: Feature = 1 << 14;
pub const FEATURE_SELECTION_RANGES: Feature = 1 << 15;
pub const FEATURE_LINKED_EDITING: Feature = 1 << 16;
pub const FEATURE_AUTO_INSERT: Feature = 1 << 17;
pub const FEATURE_DOCUMENT_SYMBOLS: Feature = 1 << 18;
pub const FEATURE_CODE_LENS: Feature = 1 << 19;
pub const FEATURE_NONE: Feature = 0;
pub const FEATURE_ALL: Feature = (FEATURE_CODE_LENS << 1) - 1;

const FEATURE_MASK: Feature = FEATURE_ALL;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fidelity {
    Exact,
    Atom,
    Approximate,
    None,
}

impl Fidelity {
    pub fn is_exact(self) -> bool { ::tsox_core::fntrace::enter("is_exact"); 
        self == Fidelity::Exact
    }

    pub fn is_single_segment(self) -> bool { ::tsox_core::fntrace::enter("is_single_segment"); 
        self == Fidelity::Exact || self == Fidelity::Atom
    }

    pub fn is_none(self) -> bool { ::tsox_core::fntrace::enter("is_none"); 
        self == Fidelity::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Segment {
    pub virtual_start: TextPos,
    pub virtual_end: TextPos,
    pub original_start: TextPos,
    pub original_end: TextPos,
    pub kind: Kind,
    pub features: Feature,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MappedPosition {
    pub position: TextPos,
    pub fidelity: Fidelity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MappedSpan {
    pub span: TextRange,
    pub fidelity: Fidelity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingErrorKind {
    Overlap,
    OutOfBounds,
    VerbatimMismatch,
    Kind,
    Feature,
}

#[derive(Debug, Clone, Copy)]
pub struct MappingError {
    pub kind: MappingErrorKind,
    pub virtual_pos: TextPos,
    pub original_pos: TextPos,
}

impl MappingError {
    pub fn error(&self) -> String { ::tsox_core::fntrace::enter("error"); 
        match self.kind {
            MappingErrorKind::Overlap => format!(
                "content mapper position mappings overlap or are out of order near virtual offset {}",
                self.virtual_pos
            ),
            MappingErrorKind::OutOfBounds => format!(
                "content mapper position mapping points outside the original content at original offset {}",
                self.original_pos
            ),
            MappingErrorKind::VerbatimMismatch => format!(
                "content mapper verbatim mapping does not match the original content at virtual offset {}, original offset {}",
                self.virtual_pos, self.original_pos
            ),
            MappingErrorKind::Kind => format!(
                "content mapper position mapping has an invalid kind at virtual offset {}",
                self.virtual_pos
            ),
            MappingErrorKind::Feature => format!(
                "content mapper position mappings have invalid features near original offset {}",
                self.original_pos
            ),
        }
    }
}

pub struct SpanMap {
    segments: Vec<Segment>,
    original_index: OnceLock<OriginalIndex>,
}

impl SpanMap {
    pub fn validate(&self, virtual_text: &str, original: &str) -> Option<MappingError> { ::tsox_core::fntrace::enter("validate"); 
        let virtual_len = virtual_text.len();
        let original_len = original.len();
        let mut previous_virtual_end: TextPos = 0;
        for s in &self.segments {
            if s.virtual_start < previous_virtual_end
                || s.virtual_end < s.virtual_start
                || s.virtual_end > virtual_len
            {
                return Some(MappingError {
                    kind: MappingErrorKind::Overlap,
                    virtual_pos: s.virtual_start,
                    original_pos: 0,
                });
            }
            previous_virtual_end = s.virtual_end;
            if s.original_end < s.original_start || s.original_end > original_len {
                return Some(MappingError {
                    kind: MappingErrorKind::OutOfBounds,
                    virtual_pos: s.virtual_start,
                    original_pos: s.original_end,
                });
            }
            if s.kind != Kind::Verbatim
                && s.kind != Kind::Atom
                && s.kind != Kind::Alias
            {
                return Some(MappingError {
                    kind: MappingErrorKind::Kind,
                    virtual_pos: s.virtual_start,
                    original_pos: s.original_start,
                });
            }
            if s.kind == Kind::Verbatim {
                if s.virtual_end - s.virtual_start != s.original_end - s.original_start
                    || virtual_text[s.virtual_start..s.virtual_end]
                        != original[s.original_start..s.original_end]
                {
                    return Some(MappingError {
                        kind: MappingErrorKind::VerbatimMismatch,
                        virtual_pos: s.virtual_start,
                        original_pos: s.original_start,
                    });
                }
            }
            if s.features & !FEATURE_MASK != 0 {
                return Some(MappingError {
                    kind: MappingErrorKind::Feature,
                    virtual_pos: s.virtual_start,
                    original_pos: s.original_start,
                });
            }
        }
        None
    }

    pub fn segments(&self) -> Vec<Segment> { ::tsox_core::fntrace::enter("segments"); 
        self.segments.clone()
    }

    pub fn virtual_to_original_span(&self, r: TextRange) -> (TextRange, Fidelity) { ::tsox_core::fntrace::enter("virtual_to_original_span"); 
        let virtual_start = r.pos() as TextPos;
        let virtual_end = std::cmp::max(r.end() as TextPos, virtual_start);
        if virtual_start == virtual_end {
            let (position, fidelity) = self.virtual_to_original_position(virtual_start);
            return (
                TextRange::new(position, position),
                fidelity,
            );
        }
        let (start_idx, start_in) = self.segment_index_at(virtual_start);
        let end_probe = virtual_end - 1;
        let (end_idx, end_in) = self.segment_index_at(end_probe);
        if start_idx == end_idx && start_in == end_in {
            if start_in {
                let seg = &self.segments[start_idx as usize];
                if seg.kind == Kind::Verbatim {
                    let orig_start = clamp(
                        seg.original_start + (virtual_start - seg.virtual_start),
                        seg.original_start,
                        seg.original_end,
                    );
                    let orig_end = clamp(
                        seg.original_start + (virtual_end - seg.virtual_start),
                        orig_start,
                        seg.original_end,
                    );
                    return (
                        TextRange::new(orig_start, orig_end),
                        Fidelity::Exact,
                    );
                }
                return (
                    TextRange::new(seg.original_start, seg.original_end),
                    Fidelity::Atom,
                );
            }
            let pos = self.insertion_point(start_idx);
            return (TextRange::new(pos, pos), Fidelity::None);
        }
        let orig_start = self.map_low(virtual_start, start_idx, start_in);
        let orig_end = std::cmp::max(self.map_high(virtual_end, end_idx, end_in), orig_start);
        (
            TextRange::new(orig_start, orig_end),
            Fidelity::Approximate,
        )
    }

    pub fn virtual_to_original_span_for_feature(
        &self,
        r: TextRange,
        feature: Feature,
    ) -> (TextRange, Fidelity) { ::tsox_core::fntrace::enter("virtual_to_original_span_for_feature"); 
        let mapped = self.virtual_to_original_span(r);
        if self.virtual_span_supports_feature(r, feature) {
            return mapped;
        }
        (mapped.0, Fidelity::None)
    }

    fn virtual_span_supports_feature(&self, r: TextRange, feature: Feature) -> bool { ::tsox_core::fntrace::enter("virtual_span_supports_feature"); 
        let start = r.pos() as TextPos;
        let end = std::cmp::max(r.end() as TextPos, start);
        if start == end {
            let (index, inside) = self.segment_index_at(start);
            return inside && supports_feature(self.segments[index as usize], feature);
        }
        let (index, inside) = self.segment_index_at(start);
        if !inside {
            return false;
        }
        let mut index = index as usize;
        let mut covered_through = start;
        while index < self.segments.len() && covered_through < end {
            let segment = self.segments[index as usize];
            if segment.virtual_start > covered_through
                || segment.virtual_end <= covered_through
                || !supports_feature(segment, feature)
            {
                return false;
            }
            covered_through = segment.virtual_end;
            index += 1;
        }
        covered_through >= end
    }

    pub fn virtual_to_original_position(&self, pos: TextPos) -> (TextPos, Fidelity) { ::tsox_core::fntrace::enter("virtual_to_original_position"); 
        let (idx, in_seg) = self.segment_index_at(pos);
        if !in_seg {
            return (self.insertion_point(idx), Fidelity::None);
        }
        let seg = &self.segments[idx as usize];
        if seg.kind == Kind::Verbatim {
            return (
                clamp(
                    seg.original_start + (pos - seg.virtual_start),
                    seg.original_start,
                    seg.original_end,
                ),
                Fidelity::Exact,
            );
        }
        (seg.original_start, Fidelity::Atom)
    }

    pub fn virtual_to_original_position_exact(&self, pos: TextPos) -> (TextPos, bool) { ::tsox_core::fntrace::enter("virtual_to_original_position_exact"); 
        let (mapped, fidelity) = self.virtual_to_original_position(pos);
        if fidelity != Fidelity::Exact {
            return (mapped, false);
        }
        let (index, inside) = self.segment_index_at(pos);
        if !inside || self.segments[index as usize].kind != Kind::Verbatim {
            return (mapped, false);
        }
        if index > 0 {
            let previous = self.segments[(index - 1) as usize];
            if previous.virtual_end == pos
                && (previous.kind != Kind::Verbatim
                    || previous.original_end != self.segments[index as usize].original_start)
            {
                return (mapped, false);
            }
        }
        (mapped, true)
    }

    pub fn virtual_to_original_position_for_feature(
        &self,
        pos: TextPos,
        feature: Feature,
    ) -> (TextPos, Fidelity) { ::tsox_core::fntrace::enter("virtual_to_original_position_for_feature"); 
        let mapped = self.virtual_to_original_position(pos);
        let (index, inside) = self.segment_index_at(pos);
        if !inside || !supports_feature(self.segments[index as usize], feature) {
            return (mapped.0, Fidelity::None);
        }
        mapped
    }

    pub fn alias_for_virtual_span(&self, r: TextRange) -> Option<Segment> { ::tsox_core::fntrace::enter("alias_for_virtual_span"); 
        let (index, inside) = self.segment_index_at(r.pos() as TextPos);
        if !inside {
            return None;
        }
        let segment = self.segments[index as usize];
        if segment.kind == Kind::Alias
            && r.pos() == segment.virtual_start
            && r.end() == segment.virtual_end
        {
            Some(segment)
        } else {
            None
        }
    }

    fn segment_index_at(&self, pos: TextPos) -> (isize, bool) { ::tsox_core::fntrace::enter("segment_index_at"); 
        let idx = self
            .segments
            .partition_point(|s| s.virtual_start < pos);
        if idx < self.segments.len() && self.segments[idx].virtual_start == pos {
            return (idx as isize, true);
        }
        let prev = idx as isize - 1;
        let contained = prev >= 0
            && (pos < self.segments[prev as usize].virtual_end
                || (prev as usize == self.segments.len() - 1
                    && pos == self.segments[prev as usize].virtual_end));
        (prev, contained)
    }

    fn insertion_point(&self, prev: isize) -> TextPos { ::tsox_core::fntrace::enter("insertion_point"); 
        if prev < 0 {
            return 0;
        }
        self.segments[prev as usize].original_end
    }

    fn map_low(&self, pos: TextPos, idx: isize, in_seg: bool) -> TextPos { ::tsox_core::fntrace::enter("map_low"); 
        if !in_seg {
            return self.insertion_point(idx);
        }
        let seg = &self.segments[idx as usize];
        if seg.kind == Kind::Verbatim {
            clamp(
                seg.original_start + (pos - seg.virtual_start),
                seg.original_start,
                seg.original_end,
            )
        } else {
            seg.original_start
        }
    }

    fn map_high(&self, pos: TextPos, idx: isize, in_seg: bool) -> TextPos { ::tsox_core::fntrace::enter("map_high"); 
        if !in_seg {
            return self.insertion_point(idx);
        }
        let seg = &self.segments[idx as usize];
        if seg.kind == Kind::Verbatim {
            clamp(
                seg.original_start + (pos - seg.virtual_start),
                seg.original_start,
                seg.original_end,
            )
        } else {
            seg.original_end
        }
    }

    fn original_index(&self) -> &OriginalIndex { ::tsox_core::fntrace::enter("original_index"); 
        self.original_index.get_or_init(|| {
            let mut segments = self.segments.clone();
            segments.sort_by(|a, b| {
                a.original_start
                    .cmp(&b.original_start)
                    .then(a.original_end.cmp(&b.original_end))
                    .then(a.virtual_start.cmp(&b.virtual_start))
            });
            let mut leaf_count = 1usize;
            while leaf_count < segments.len() {
                leaf_count *= 2;
            }
            let mut max_ends = vec![0usize; 2 * leaf_count];
            for (i, segment) in segments.iter().enumerate() {
                max_ends[leaf_count + i] = segment.original_end;
            }
            for i in (1..leaf_count).rev() {
                max_ends[i] = std::cmp::max(max_ends[2 * i], max_ends[2 * i + 1]);
            }
            OriginalIndex {
                segments,
                leaf_count,
                max_ends,
            }
        })
    }
}

pub fn new_span_map(mut segments: Vec<Segment>) -> SpanMap { ::tsox_core::fntrace::enter("new_span_map"); 
    segments.sort_by(|a, b| a.virtual_start.cmp(&b.virtual_start));
    SpanMap {
        segments,
        original_index: OnceLock::new(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OriginalIndex {
    segments: Vec<Segment>,
    leaf_count: usize,
    max_ends: Vec<TextPos>,
}

impl OriginalIndex {
    pub fn segments_at_original_position(&self, pos: TextPos) -> (Vec<Segment>, bool) { ::tsox_core::fntrace::enter("segments_at_original_position"); 
        let start = self
            .segments
            .partition_point(|s| s.original_start < pos);
        let mut results = self.segments_ending_after_position(start, pos);
        let end = self
            .segments
            .partition_point(|s| s.original_start <= pos);
        results.extend_from_slice(&self.segments[start..end]);
        let has = !results.is_empty();
        (results, has)
    }

    pub fn segments_ending_after_position(&self, limit: usize, pos: TextPos) -> Vec<Segment> { ::tsox_core::fntrace::enter("segments_ending_after_position"); 
        let mut results = Vec::new();
        self.collect_segments_ending_at_or_after(
            1,
            0,
            self.leaf_count,
            limit,
            pos,
            false,
            &mut results,
        );
        results
    }

    fn collect_segments_ending_at_or_after(
        &self,
        node: usize,
        start: usize,
        end: usize,
        limit: usize,
        pos: TextPos,
        include_end: bool,
        results: &mut Vec<Segment>,
    ) { ::tsox_core::fntrace::enter("collect_segments_ending_at_or_after"); 
        if start >= limit
            || self.max_ends[node] < pos
            || (!include_end && self.max_ends[node] == pos)
        {
            return;
        }
        if end - start == 1 {
            results.push(self.segments[start]);
            return;
        }
        let middle = start + (end - start) / 2;
        self.collect_segments_ending_at_or_after(
            2 * node,
            start,
            middle,
            limit,
            pos,
            include_end,
            results,
        );
        self.collect_segments_ending_at_or_after(
            2 * node + 1,
            middle,
            end,
            limit,
            pos,
            include_end,
            results,
        );
    }

    pub fn segment_groups_at_original_position(
        &self,
        pos: TextPos,
    ) -> Vec<SegmentGroupAtOriginalPosition> { ::tsox_core::fntrace::enter("segment_groups_at_original_position"); 
        let limit = self
            .segments
            .partition_point(|s| s.original_start <= pos);
        let mut segments = Vec::new();
        self.collect_segments_ending_at_or_after(1, 0, self.leaf_count, limit, pos, true, &mut segments);
        let mut groups = Vec::new();
        let mut start = 0;
        while start < segments.len() {
            let mut end = start + 1;
            while end < segments.len() && same_original_range(&segments[start], &segments[end]) {
                end += 1;
            }
            let segment = segments[start];
            if pos <= segment.original_end {
                groups.push(SegmentGroupAtOriginalPosition {
                    segments: segments[start..end].to_vec(),
                    at_end: pos == segment.original_end && pos != segment.original_start,
                });
            }
            start = end;
        }
        groups
    }
}

pub struct SegmentGroupAtOriginalPosition {
    pub segments: Vec<Segment>,
    pub at_end: bool,
}

pub fn same_original_range(left: &Segment, right: &Segment) -> bool { ::tsox_core::fntrace::enter("same_original_range"); 
    left.original_start == right.original_start && left.original_end == right.original_end
}

pub fn supports_feature(segment: Segment, feature: Feature) -> bool { ::tsox_core::fntrace::enter("supports_feature"); 
    segment.features & feature != 0
}

pub fn clamp(v: TextPos, lo: TextPos, hi: TextPos) -> TextPos { ::tsox_core::fntrace::enter("clamp"); 
    std::cmp::max(lo, std::cmp::min(v, hi))
}

impl Ord for Segment {
    fn cmp(&self, other: &Self) -> Ordering { ::tsox_core::fntrace::enter("cmp"); 
        self.virtual_start.cmp(&other.virtual_start)
    }
}

impl PartialOrd for Segment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { ::tsox_core::fntrace::enter("partial_cmp"); 
        Some(self.cmp(other))
    }
}
