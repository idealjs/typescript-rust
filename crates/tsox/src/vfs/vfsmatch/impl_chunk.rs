use super::*;

impl GlobPattern {
    pub fn matches(&self, path: &str) -> bool {
        self.match_path_parts(path, "", 0, 0, false)
    }

    pub fn matches_parts(&self, prefix: &str, suffix: &str) -> bool {
        self.match_path_parts(prefix, suffix, 0, 0, false)
    }

    pub fn matches_prefix_parts(&self, prefix: &str, suffix: &str) -> bool {
        self.match_path_parts(prefix, suffix, 0, 0, true)
    }

    pub(crate) fn match_path_parts(
        &self,
        prefix: &str,
        suffix: &str,
        path_offset: usize,
        comp_idx: usize,
        prefix_only: bool,
    ) -> bool {
        let mut path_offset = path_offset;
        let mut comp_idx = comp_idx;

        loop {
            let (path_part, next_offset, ok) = next_path_part_parts(prefix, suffix, path_offset);
            if !ok {
                if prefix_only {
                    return true;
                }
                return self.pattern_satisfied(comp_idx);
            }

            if comp_idx >= self.components.len() {
                return self.is_exclude && !prefix_only;
            }

            let comp = &self.components[comp_idx];
            match comp.kind {
                ComponentKind::DoubleAsterisk => {
                    if self.match_path_parts(prefix, suffix, path_offset, comp_idx + 1, prefix_only)
                    {
                        return true;
                    }
                    if !self.is_exclude
                        && (is_hidden_path(&path_part) || is_package_folder(&path_part))
                    {
                        return false;
                    }
                    path_offset = next_offset;
                    continue;
                }
                ComponentKind::Literal => {
                    if !self.strings_equal(&comp.literal, &path_part) {
                        return false;
                    }
                }
                ComponentKind::Wildcard => {
                    if comp.skip_package_folders && is_package_folder(&path_part) {
                        return false;
                    }
                    if !self.match_wildcard(&comp.segments, &path_part) {
                        return false;
                    }
                }
            }

            path_offset = next_offset;
            comp_idx += 1;
        }
    }

    pub(crate) fn pattern_satisfied(&self, comp_idx: usize) -> bool {
        for c in &self.components[comp_idx..] {
            if c.kind != ComponentKind::DoubleAsterisk {
                return false;
            }
        }
        true
    }

    pub(crate) fn match_wildcard(&self, segs: &[Segment], s: &str) -> bool {
        if !self.is_exclude
            && !segs.is_empty()
            && is_hidden_path(s)
            && (segs[0].kind == SegmentKind::Star || segs[0].kind == SegmentKind::Question)
        {
            return false;
        }

        if segs.len() == 2
            && segs[0].kind == SegmentKind::Star
            && segs[1].kind == SegmentKind::Literal
        {
            let suffix = segs[1].literal.as_bytes();
            let s_bytes = s.as_bytes();
            if s_bytes.len() < suffix.len()
                || !self.bytes_equal(suffix, &s_bytes[s_bytes.len() - suffix.len()..])
            {
                return false;
            }
            return self.should_include_min_js(s, segs);
        }

        self.match_segments(segs, s) && self.should_include_min_js(s, segs)
    }

    pub(crate) fn match_segments(&self, segs: &[Segment], s: &str) -> bool {
        let mut seg_idx: i32 = 0;
        let mut s_idx: usize = 0;
        let mut star_seg_idx: i32 = -1;
        let mut star_s_idx: usize = 0;

        while s_idx < s.len() {
            if (seg_idx as usize) < segs.len() {
                let seg = &segs[seg_idx as usize];
                match seg.kind {
                    SegmentKind::Literal => {
                        let lit = seg.literal.as_bytes();
                        let end = s_idx + lit.len();
                        if end <= s.len() && self.bytes_equal(lit, &s.as_bytes()[s_idx..end]) {
                            s_idx = end;
                            seg_idx += 1;
                            continue;
                        }
                    }
                    SegmentKind::Question => {
                        if s.as_bytes()[s_idx] != b'/' {
                            let size = next_rune_size(s, s_idx);
                            s_idx += size;
                            seg_idx += 1;
                            continue;
                        }
                    }
                    SegmentKind::Star => {
                        star_seg_idx = seg_idx;
                        star_s_idx = s_idx;
                        seg_idx += 1;
                        continue;
                    }
                }
            }

            if star_seg_idx >= 0 && star_s_idx < s.len() && s.as_bytes()[star_s_idx] != b'/' {
                let size = next_rune_size(s, star_s_idx);
                star_s_idx += size;
                s_idx = star_s_idx;
                seg_idx = star_seg_idx + 1;
                continue;
            }

            return false;
        }

        while (seg_idx as usize) < segs.len() && segs[seg_idx as usize].kind == SegmentKind::Star {
            seg_idx += 1;
        }
        (seg_idx as usize) >= segs.len()
    }

    pub(crate) fn should_include_min_js(&self, filename: &str, segs: &[Segment]) -> bool {
        if !self.exclude_min_js {
            return true;
        }
        if !self.has_min_js_suffix(filename) {
            return true;
        }

        if self.pattern_mentions_min_suffix(segs) {
            return true;
        }
        false
    }

    pub(crate) fn has_min_js_suffix(&self, filename: &str) -> bool {
        const MIN_JS: &str = ".min.js";
        if self.case_sensitive {
            filename.ends_with(MIN_JS)
        } else {
            filename.len() >= MIN_JS.len()
                && filename[filename.len() - MIN_JS.len()..].eq_ignore_ascii_case(MIN_JS)
        }
    }

    pub(crate) fn pattern_mentions_min_suffix(&self, segs: &[Segment]) -> bool {
        for seg in segs {
            if seg.kind != SegmentKind::Literal {
                continue;
            }
            let lit = if self.case_sensitive {
                seg.literal.as_str()
            } else {
                if seg.literal.to_ascii_lowercase().contains(".min.js")
                    || seg.literal.to_ascii_lowercase().contains(".min.")
                {
                    return true;
                }
                continue;
            };
            if lit.contains(".min.js") || lit.contains(".min.") {
                return true;
            }
        }
        false
    }

    pub(crate) fn strings_equal(&self, a: &str, b: &str) -> bool {
        if self.case_sensitive {
            a == b
        } else {
            a.eq_ignore_ascii_case(b)
        }
    }

    pub(crate) fn bytes_equal(&self, a: &[u8], b: &[u8]) -> bool {
        if self.case_sensitive {
            a == b
        } else {
            a.eq_ignore_ascii_case(b)
        }
    }
}

pub(crate) fn next_rune_size(s: &str, idx: usize) -> usize {
    s[idx..].chars().next().map_or(0, |c| c.len_utf8())
}

pub(crate) fn next_path_part_single(s: &str, offset: usize) -> (String, usize, bool) {
    if offset >= s.len() {
        return (String::new(), offset, false);
    }
    let bytes = s.as_bytes();
    if offset == 0 && !s.is_empty() && bytes[0] == b'/' {
        return (String::new(), 1, true);
    }
    let mut offset = offset;
    while offset < s.len() && bytes[offset] == b'/' {
        offset += 1;
    }
    if offset >= s.len() {
        return (String::new(), offset, false);
    }
    let rest = &s[offset..];
    if let Some(idx) = rest.find('/') {
        (rest[..idx].to_string(), offset + idx, true)
    } else {
        (rest.to_string(), s.len(), true)
    }
}

pub fn next_path_part_parts(prefix: &str, suffix: &str, offset: usize) -> (String, usize, bool) {
    if suffix.is_empty() {
        return next_path_part_single(prefix, offset);
    }
    if prefix.is_empty() {
        return next_path_part_single(suffix, offset);
    }

    let total_len = prefix.len() + suffix.len();
    if offset >= total_len {
        return (String::new(), offset, false);
    }

    if offset == 0 && !prefix.is_empty() && prefix.as_bytes()[0] == b'/' {
        return (String::new(), 1, true);
    }

    if offset < prefix.len() {
        let mut o = offset;
        while o < prefix.len() && prefix.as_bytes()[o] == b'/' {
            o += 1;
        }
        if o < prefix.len() {
            let rest = &prefix[o..];
            let idx = rest.find('/').unwrap_or(rest.len());
            return (rest[..idx].to_string(), o + idx, true);
        }
    }

    let s_off = offset.saturating_sub(prefix.len());
    if s_off >= suffix.len() {
        return (String::new(), offset, false);
    }
    (suffix[s_off..].to_string(), total_len, true)
}

pub(crate) fn is_hidden_path(name: &str) -> bool {
    !name.is_empty() && name.as_bytes()[0] == b'.'
}
