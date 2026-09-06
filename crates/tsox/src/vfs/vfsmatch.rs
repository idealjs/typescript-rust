use crate::tspath;
use crate::vfs::FS;
use std::collections::HashSet;

pub const UNLIMITED_DEPTH: i32 = i32::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Usage {
    Files,
    Directories,
    Exclude,
}

pub fn read_directory(
    host: &dyn FS,
    current_dir: &str,
    path: &str,
    extensions: &[&str],
    excludes: &[&str],
    includes: &[&str],
    depth: i32,
) -> Vec<String> {
    match_files(
        path,
        extensions,
        excludes,
        includes,
        host.use_case_sensitive_file_names(),
        current_dir,
        depth,
        host,
    )
}

pub fn is_implicit_glob(last_path_component: &str) -> bool {
    !last_path_component.contains('.')
        && !last_path_component.contains('*')
        && !last_path_component.contains('?')
}

const WILDCARD_CHARS: &[char] = &['*', '?'];

fn get_include_base_path(absolute: &str) -> String {
    let wildcard_offset = absolute.find(|c: char| WILDCARD_CHARS.contains(&c));
    match wildcard_offset {
        None => {

            if !tspath::has_extension(absolute) {
                absolute.to_string()
            } else {
                tspath::remove_trailing_directory_separator(&tspath::get_directory_path(absolute))
            }
        }
        Some(woff) => {
            let prefix = &absolute[..woff];
            let last_slash = prefix.rfind('/').map_or(0, |i| i);
            absolute[..last_slash].to_string()
        }
    }
}

pub fn get_base_paths(
    path: &str,
    includes: &[&str],
    use_case_sensitive_file_names: bool,
) -> Vec<String> {
    let mut base_paths: Vec<String> = vec![path.to_string()];

    if !includes.is_empty() {
        let options = tspath::ComparePathsOptions {
            current_directory: path.to_string(),
            use_case_sensitive_file_names,
        };

        let mut include_base_paths: Vec<String> = Vec::new();
        for include in includes {
            let absolute = if tspath::is_rooted_disk_path(include) {
                include.to_string()
            } else {
                tspath::normalize_path(&tspath::combine_paths(path, &[include]))
            };
            include_base_paths.push(get_include_base_path(&absolute));
        }

        let case_sensitive = use_case_sensitive_file_names;
        include_base_paths.sort_by(|a, b| {
            if case_sensitive {
                a.cmp(b)
            } else {
                a.to_ascii_lowercase().cmp(&b.to_ascii_lowercase())
            }
        });

        for include_base_path in &include_base_paths {
            let is_new = base_paths
                .iter()
                .all(|bp| !contains_path(bp, include_base_path, &options));
            if is_new {
                base_paths.push(include_base_path.clone());
            }
        }
    }

    base_paths
}

fn contains_path(parent: &str, child: &str, options: &tspath::ComparePathsOptions) -> bool {
    let parent_components = tspath::reduce_path_components(&tspath::get_path_components(
        parent,
        &options.current_directory,
    ));
    let child_components = tspath::reduce_path_components(&tspath::get_path_components(
        child,
        &options.current_directory,
    ));

    if child_components.len() < parent_components.len() {
        return false;
    }

    let case_sensitive = options.use_case_sensitive_file_names;
    for (i, pc) in parent_components.iter().enumerate() {
        let cc = &child_components[i];
        if i == 0 {

            if !pc.eq_ignore_ascii_case(cc) {
                return false;
            }
        } else if case_sensitive {
            if pc != cc {
                return false;
            }
        } else if !pc.eq_ignore_ascii_case(cc) {
            return false;
        }
    }
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ComponentKind {
    Literal,
    Wildcard,
    DoubleAsterisk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SegmentKind {
    Literal,
    Star,
    Question,
}

#[derive(Debug, Clone)]
struct Segment {
    kind: SegmentKind,
    literal: String,
}

#[derive(Debug, Clone)]
struct Component {
    kind: ComponentKind,
    literal: String,
    segments: Vec<Segment>,
    skip_package_folders: bool,
}

#[derive(Debug, Clone)]
pub struct GlobPattern {
    components: Vec<Component>,
    is_exclude: bool,
    case_sensitive: bool,
    exclude_min_js: bool,
}

pub fn compile_glob_pattern(
    spec: &str,
    base_path: &str,
    usage: Usage,
    case_sensitive: bool,
) -> Option<GlobPattern> {
    let mut parts = get_normalized_path_components(spec, base_path);

    if usage != Usage::Exclude {
        if let Some(last) = parts.last() {
            if last == "**" {
                return None;
            }
        }
    }

    if let Some(first) = parts.first_mut() {
        *first = tspath::remove_trailing_directory_separator(first);
    }

    if let Some(last) = parts.last() {
        if is_implicit_glob(last) {
            parts.push("**".to_string());
            parts.push("*".to_string());
        }
    }

    let is_include = usage != Usage::Exclude;
    let mut components = Vec::with_capacity(parts.len());
    for part in &parts {
        components.push(parse_component(part, is_include));
    }

    Some(GlobPattern {
        components,
        is_exclude: usage == Usage::Exclude,
        case_sensitive,
        exclude_min_js: usage == Usage::Files,
    })
}

fn get_normalized_path_components(path: &str, current_directory: &str) -> Vec<String> {
    let combined = tspath::combine_paths(current_directory, &[path]);
    let normalized = tspath::normalize_path(&combined);
    tspath::reduce_path_components(&tspath::get_path_components(&normalized, ""))
}

fn parse_component(s: &str, is_include: bool) -> Component {
    if s == "**" {
        return Component {
            kind: ComponentKind::DoubleAsterisk,
            literal: String::new(),
            segments: Vec::new(),
            skip_package_folders: false,
        };
    }
    if !s.contains('*') && !s.contains('?') {
        return Component {
            kind: ComponentKind::Literal,
            literal: s.to_string(),
            segments: Vec::new(),
            skip_package_folders: false,
        };
    }
    Component {
        kind: ComponentKind::Wildcard,
        literal: String::new(),
        segments: parse_segments(s),
        skip_package_folders: is_include,
    }
}

fn parse_segments(s: &str) -> Vec<Segment> {
    let wildcards = s.bytes().filter(|&b| b == b'*' || b == b'?').count();
    let mut result = Vec::with_capacity(2 * wildcards + 1);
    let mut start = 0usize;
    for (i, b) in s.bytes().enumerate() {
        match b {
            b'*' | b'?' => {
                if i > start {
                    result.push(Segment {
                        kind: SegmentKind::Literal,
                        literal: s[start..i].to_string(),
                    });
                }
                result.push(Segment {
                    kind: if b == b'*' {
                        SegmentKind::Star
                    } else {
                        SegmentKind::Question
                    },
                    literal: String::new(),
                });
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < s.len() {
        result.push(Segment {
            kind: SegmentKind::Literal,
            literal: s[start..].to_string(),
        });
    }
    result
}

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

    fn match_path_parts(
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

    fn pattern_satisfied(&self, comp_idx: usize) -> bool {
        for c in &self.components[comp_idx..] {
            if c.kind != ComponentKind::DoubleAsterisk {
                return false;
            }
        }
        true
    }

    fn match_wildcard(&self, segs: &[Segment], s: &str) -> bool {

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

    fn match_segments(&self, segs: &[Segment], s: &str) -> bool {
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

    fn should_include_min_js(&self, filename: &str, segs: &[Segment]) -> bool {
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

    fn has_min_js_suffix(&self, filename: &str) -> bool {
        const MIN_JS: &str = ".min.js";
        if self.case_sensitive {
            filename.ends_with(MIN_JS)
        } else {
            filename.len() >= MIN_JS.len()
                && filename[filename.len() - MIN_JS.len()..].eq_ignore_ascii_case(MIN_JS)
        }
    }

    fn pattern_mentions_min_suffix(&self, segs: &[Segment]) -> bool {
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

    fn strings_equal(&self, a: &str, b: &str) -> bool {
        if self.case_sensitive {
            a == b
        } else {
            a.eq_ignore_ascii_case(b)
        }
    }

    fn bytes_equal(&self, a: &[u8], b: &[u8]) -> bool {
        if self.case_sensitive {
            a == b
        } else {
            a.eq_ignore_ascii_case(b)
        }
    }
}

fn next_rune_size(s: &str, idx: usize) -> usize {
    s[idx..].chars().next().map_or(0, |c| c.len_utf8())
}

fn next_path_part_single(s: &str, offset: usize) -> (String, usize, bool) {
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

fn is_hidden_path(name: &str) -> bool {
    !name.is_empty() && name.as_bytes()[0] == b'.'
}

fn is_package_folder(name: &str) -> bool {
    name.eq_ignore_ascii_case("node_modules")
        || name.eq_ignore_ascii_case("jspm_packages")
        || name.eq_ignore_ascii_case("bower_components")
}

pub fn ensure_trailing_slash(s: &str) -> String {
    if !s.is_empty() && s.as_bytes()[s.len() - 1] != b'/' {
        format!("{s}/")
    } else {
        s.to_string()
    }
}

struct GlobMatcher {
    includes: Vec<GlobPattern>,
    excludes: Vec<GlobPattern>,
    had_includes: bool,
}

fn new_glob_matcher(
    include_specs: &[&str],
    exclude_specs: &[&str],
    base_path: &str,
    case_sensitive: bool,
    usage: Usage,
) -> GlobMatcher {
    let mut includes = Vec::with_capacity(include_specs.len());
    for spec in include_specs {
        if let Some(p) = compile_glob_pattern(spec, base_path, usage, case_sensitive) {
            includes.push(p);
        }
    }
    let mut excludes = Vec::with_capacity(exclude_specs.len());
    for spec in exclude_specs {
        if let Some(p) = compile_glob_pattern(spec, base_path, Usage::Exclude, case_sensitive) {
            excludes.push(p);
        }
    }
    GlobMatcher {
        includes,
        excludes,
        had_includes: !include_specs.is_empty(),
    }
}

impl GlobMatcher {

    fn matches_file_parts(&self, prefix: &str, suffix: &str) -> (usize, bool) {
        for ex in &self.excludes {
            if ex.matches_parts(prefix, suffix) {
                return (0, false);
            }
        }
        if self.includes.is_empty() {
            if self.had_includes {
                return (0, false);
            }
            return (0, true);
        }
        for (i, inc) in self.includes.iter().enumerate() {
            if inc.matches_parts(prefix, suffix) {
                return (i, true);
            }
        }
        (0, false)
    }

    fn matches_directory_parts(&self, prefix: &str, suffix: &str) -> bool {
        for ex in &self.excludes {
            if ex.matches_parts(prefix, suffix) {
                return false;
            }
        }
        if self.includes.is_empty() {
            return !self.had_includes;
        }
        for inc in &self.includes {
            if inc.matches_prefix_parts(prefix, suffix) {
                return true;
            }
        }
        false
    }
}

struct GlobVisitor<'a> {
    host: &'a dyn FS,
    file_matcher: GlobMatcher,
    directory_matcher: GlobMatcher,
    extensions: &'a [&'a str],
    use_case_sensitive_file_names: bool,
    visited: HashSet<String>,
    results: Vec<Vec<String>>,
}

impl<'a> GlobVisitor<'a> {
    fn visit(&mut self, path: &str, absolute_path: &str, depth: i32, resolved_real_path: &str) {

        let real_path = if !resolved_real_path.is_empty() {
            resolved_real_path.to_string()
        } else {
            self.host.realpath(absolute_path)
        };
        let canonical_path =
            tspath::get_canonical_file_name(&real_path, self.use_case_sensitive_file_names);
        if self.visited.contains(&canonical_path) {
            return;
        }
        self.visited.insert(canonical_path);

        let entries = self.host.get_accessible_entries(absolute_path);

        let path_prefix = ensure_trailing_slash(path);
        let abs_prefix = ensure_trailing_slash(absolute_path);

        for file in &entries.files {
            if !self.extensions.is_empty()
                && !tspath::file_extension_is_one_of(file, self.extensions)
            {
                continue;
            }
            let (idx, ok) = self.file_matcher.matches_file_parts(&abs_prefix, file);
            if ok {
                if idx < self.results.len() {
                    self.results[idx].push(format!("{path_prefix}{file}"));
                }
            }
        }

        let mut depth = depth;
        if depth != UNLIMITED_DEPTH {
            depth -= 1;
            if depth == 0 {
                return;
            }
        }

        for dir in &entries.directories {
            if !self
                .directory_matcher
                .matches_directory_parts(&abs_prefix, dir)
            {
                continue;
            }
            let abs_dir = format!("{abs_prefix}{dir}");

            let is_symlink = entries.symlinks.iter().any(|s| s == dir);
            let child_real_path = if !is_symlink {
                tspath::combine_paths(&real_path, &[dir])
            } else {
                String::new()
            };
            self.visit(
                &format!("{path_prefix}{dir}"),
                &abs_dir,
                depth,
                &child_real_path,
            );
        }
    }
}

pub fn match_files(
    path: &str,
    extensions: &[&str],
    excludes: &[&str],
    includes: &[&str],
    use_case_sensitive_file_names: bool,
    current_directory: &str,
    depth: i32,
    host: &dyn FS,
) -> Vec<String> {
    let path = tspath::normalize_path(path);
    let current_directory = tspath::normalize_path(current_directory);
    let absolute_path = tspath::combine_paths(&current_directory, &[&path]);

    let file_matcher = new_glob_matcher(
        includes,
        excludes,
        &absolute_path,
        use_case_sensitive_file_names,
        Usage::Files,
    );
    let directory_matcher = new_glob_matcher(
        includes,
        excludes,
        &absolute_path,
        use_case_sensitive_file_names,
        Usage::Directories,
    );

    let num_buckets = file_matcher.includes.len().max(1);
    let mut visitor = GlobVisitor {
        host,
        file_matcher,
        directory_matcher,
        extensions,
        use_case_sensitive_file_names,
        visited: HashSet::new(),
        results: vec![Vec::new(); num_buckets],
    };

    for base_path in get_base_paths(&path, includes, use_case_sensitive_file_names) {
        let abs = tspath::combine_paths(&current_directory, &[&base_path]);
        visitor.visit(&base_path, &abs, depth, "");
    }

    if visitor.results.len() == 1 {
        visitor.results.into_iter().next().unwrap()
    } else {
        visitor.results.into_iter().flatten().collect()
    }
}

pub struct SpecMatcher {
    patterns: Vec<GlobPattern>,
}

impl SpecMatcher {

    pub fn new(
        specs: &[&str],
        base_path: &str,
        usage: Usage,
        use_case_sensitive_file_names: bool,
    ) -> Option<Self> {
        if specs.is_empty() {
            return None;
        }
        let mut patterns = Vec::with_capacity(specs.len());
        for spec in specs {
            if let Some(p) =
                compile_glob_pattern(spec, base_path, usage, use_case_sensitive_file_names)
            {
                patterns.push(p);
            }
        }
        if patterns.is_empty() {
            return None;
        }
        Some(SpecMatcher { patterns })
    }

    pub fn matches(&self, path: &str) -> bool {
        self.patterns.iter().any(|p| p.matches(path))
    }

    pub fn match_index(&self, path: &str) -> i32 {
        for (i, p) in self.patterns.iter().enumerate() {
            if p.matches(path) {
                return i as i32;
            }
        }
        -1
    }
}
