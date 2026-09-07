use crate::tspath;
use crate::vfs::FS;
use std::collections::HashSet;

use super::*;

pub(crate) fn is_package_folder(name: &str) -> bool {
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

pub(crate) struct GlobMatcher {
    pub(crate) includes: Vec<GlobPattern>,
    pub(crate) excludes: Vec<GlobPattern>,
    pub(crate) had_includes: bool,
}

pub(crate) fn new_glob_matcher(
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
    pub(crate) fn matches_file_parts(&self, prefix: &str, suffix: &str) -> (usize, bool) {
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

    pub(crate) fn matches_directory_parts(&self, prefix: &str, suffix: &str) -> bool {
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

pub(crate) struct GlobVisitor<'a> {
    pub(crate) host: &'a dyn FS,
    pub(crate) file_matcher: GlobMatcher,
    pub(crate) directory_matcher: GlobMatcher,
    pub(crate) extensions: &'a [&'a str],
    pub(crate) use_case_sensitive_file_names: bool,
    pub(crate) visited: HashSet<String>,
    pub(crate) results: Vec<Vec<String>>,
}

impl<'a> GlobVisitor<'a> {
    pub(crate) fn visit(
        &mut self,
        path: &str,
        absolute_path: &str,
        depth: i32,
        resolved_real_path: &str,
    ) {
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
    pub(crate) patterns: Vec<GlobPattern>,
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
