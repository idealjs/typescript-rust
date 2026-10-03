use super::m6b_2::{clamp, supports_feature, Feature, Kind, Segment, TextPos};

pub fn original_start_projections(
    segments: &[Segment],
    start: TextPos,
    feature: Feature,
) -> Vec<TextPos> { ::tsox_core::fntrace::enter("original_start_projections"); 
    let mut results = Vec::with_capacity(segments.len());
    for segment in segments {
        if !supports_feature(*segment, feature) {
            continue;
        }
        if segment.kind == Kind::Verbatim {
            results.push(clamp(
                segment.virtual_start + (start - segment.original_start),
                segment.virtual_start,
                segment.virtual_end,
            ));
        } else {
            results.push(segment.virtual_start);
        }
    }
    results
}

pub fn original_end_projections(
    segments: &[Segment],
    end: TextPos,
    feature: Feature,
) -> Vec<TextPos> { ::tsox_core::fntrace::enter("original_end_projections"); 
    let mut results = Vec::with_capacity(segments.len());
    for segment in segments {
        if !supports_feature(*segment, feature) {
            continue;
        }
        if segment.kind == Kind::Verbatim {
            results.push(clamp(
                segment.virtual_start + (end - segment.original_start),
                segment.virtual_start,
                segment.virtual_end,
            ));
        } else {
            results.push(segment.virtual_end);
        }
    }
    results
}
