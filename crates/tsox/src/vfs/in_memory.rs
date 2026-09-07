use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

pub(super) const MAX_SYMLINK_HOPS: usize = 40;

pub struct InMemoryFS {
    pub(super) case_sensitive: bool,
    pub(super) files: RwLock<HashMap<String, String>>,
    pub(super) dirs: RwLock<std::collections::HashSet<String>>,

    pub(super) symlinks: RwLock<HashMap<String, String>>,
}

impl InMemoryFS {
    pub fn new() -> Self {
        Self::with_case_sensitivity(true)
    }

    pub fn with_case_sensitivity(case_sensitive: bool) -> Self {
        InMemoryFS {
            case_sensitive,
            files: RwLock::new(HashMap::new()),
            dirs: RwLock::new(std::collections::HashSet::new()),
            symlinks: RwLock::new(HashMap::new()),
        }
    }

    pub fn insert_file(&self, path: &str, content: &str) {
        let mut files = self.files.write().unwrap();
        let key = if self.case_sensitive {
            path.to_string()
        } else {
            let target = path.to_ascii_lowercase();
            files
                .keys()
                .find(|k| k.to_ascii_lowercase() == target)
                .cloned()
                .unwrap_or_else(|| path.to_string())
        };
        files.insert(key, content.to_string());
    }

    pub fn insert_dir(&self, path: &str) {
        let mut dirs = self.dirs.write().unwrap();

        let mut current = path.to_string();
        loop {
            if !self.case_sensitive {
                let target = current.to_ascii_lowercase();
                if let Some(existing) = dirs
                    .iter()
                    .find(|d| d.to_ascii_lowercase() == target)
                    .cloned()
                {
                    dirs.remove(&existing);
                }
            }
            dirs.insert(current.clone());
            let parent = crate::tspath::get_directory_path(&current);
            if parent == current || parent.is_empty() {
                break;
            }
            current = parent;
        }
    }

    pub(super) fn lookup_file_key(&self, path: &str) -> Option<String> {
        let files = self.files.read().unwrap();
        if files.contains_key(path) {
            return Some(path.to_string());
        }
        if self.case_sensitive {
            return None;
        }
        let target = path.to_ascii_lowercase();
        files
            .keys()
            .find(|k| k.to_ascii_lowercase() == target)
            .cloned()
    }

    pub(super) fn lookup_dir_key(&self, path: &str) -> Option<String> {
        let dirs = self.dirs.read().unwrap();
        if dirs.contains(path) {
            return Some(path.to_string());
        }
        if self.case_sensitive {
            return None;
        }
        let target = path.to_ascii_lowercase();
        dirs.iter()
            .find(|d| d.to_ascii_lowercase() == target)
            .cloned()
    }

    pub fn create_symlink(&self, link: &str, target: &str) {
        let mut symlinks = self.symlinks.write().unwrap();
        let key = if self.case_sensitive {
            link.to_string()
        } else {
            let target_lc = link.to_ascii_lowercase();
            symlinks
                .keys()
                .find(|k| k.to_ascii_lowercase() == target_lc)
                .cloned()
                .unwrap_or_else(|| link.to_string())
        };
        symlinks.insert(key, target.to_string());
    }

    pub fn read_symlink(&self, path: &str) -> Option<String> {
        self.lookup_symlink_key(path)
    }

    pub(super) fn lookup_symlink_key(&self, link: &str) -> Option<String> {
        let symlinks = self.symlinks.read().unwrap();
        if let Some(t) = symlinks.get(link) {
            return Some(t.clone());
        }
        if self.case_sensitive {
            return None;
        }
        let target = link.to_ascii_lowercase();
        symlinks
            .iter()
            .find(|(k, _)| k.to_ascii_lowercase() == target)
            .map(|(_, v)| v.clone())
    }

    pub(super) fn lookup_symlink_stored_key(&self, link: &str) -> Option<String> {
        let symlinks = self.symlinks.read().unwrap();
        if symlinks.contains_key(link) {
            return Some(link.to_string());
        }
        if self.case_sensitive {
            return None;
        }
        let target = link.to_ascii_lowercase();
        symlinks
            .iter()
            .find(|(k, _)| k.to_ascii_lowercase() == target)
            .map(|(k, _)| k.clone())
    }

    pub(super) fn is_file_path(&self, path: &str) -> bool {
        let files = self.files.read().unwrap();
        if files.contains_key(path) {
            return true;
        }
        if self.case_sensitive {
            return false;
        }
        let target = path.to_ascii_lowercase();
        files.keys().any(|k| k.to_ascii_lowercase() == target)
    }

    pub(super) fn resolve_symlinks(&self, path: &str) -> String {
        if path.is_empty() {
            return String::new();
        }
        let symlinks = self.symlinks.read().unwrap();
        if symlinks.is_empty() {
            return path.to_string();
        }
        let is_absolute = path.starts_with('/');
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut resolved = if is_absolute {
            String::from("/")
        } else {
            String::new()
        };
        let mut visited: HashSet<String> = HashSet::new();
        for part in &parts {
            if resolved.is_empty() {
                resolved.push_str(part);
            } else if resolved.ends_with('/') {
                resolved.push_str(part);
            } else {
                resolved.push('/');
                resolved.push_str(part);
            }

            let mut hops = 0;
            loop {
                hops += 1;
                if hops > MAX_SYMLINK_HOPS {
                    break;
                }
                let target = match self.symlink_target(&symlinks, &resolved) {
                    Some(t) => t,
                    None => break,
                };
                resolved = if is_absolute_path(&target) {
                    target
                } else {
                    match parent_dir(&resolved) {
                        Some(p) if p.ends_with('/') => format!("{p}{target}"),
                        Some(p) => format!("{p}/{target}"),
                        None => target,
                    }
                };
                if !visited.insert(resolved.clone()) {
                    break;
                }
            }
        }
        resolved
    }

    fn symlink_target<'a>(
        &self,
        symlinks: &'a HashMap<String, String>,
        path: &str,
    ) -> Option<String> {
        if let Some(t) = symlinks.get(path) {
            return Some(t.clone());
        }
        if self.case_sensitive {
            return None;
        }
        let target = path.to_ascii_lowercase();
        symlinks
            .iter()
            .find(|(k, _)| k.to_ascii_lowercase() == target)
            .map(|(_, v)| v.clone())
    }
}

impl Default for InMemoryFS {
    fn default() -> Self {
        Self::new()
    }
}

pub(super) fn parent_dir(path: &str) -> Option<String> {
    let trimmed = path.trim_end_matches('/');
    match trimmed.rfind('/') {
        Some(0) => Some(String::from("/")),
        Some(i) => Some(trimmed[..i].to_string()),
        None => None,
    }
}

pub(super) fn is_absolute_path(path: &str) -> bool {
    path.starts_with('/')
        || (path.len() >= 3 && path.as_bytes()[1] == b':' && path.as_bytes()[2] == b'/')
}

pub(super) fn decode_with_bom(content: &str) -> String {
    content
        .strip_prefix('\u{FEFF}')
        .map(|s| s.to_string())
        .unwrap_or_else(|| content.to_string())
}

pub(super) fn strip_path_prefix<'a>(
    haystack: &'a str,
    prefix: &str,
    case_sensitive: bool,
) -> Option<&'a str> {
    if case_sensitive {
        haystack.strip_prefix(prefix)
    } else {
        let h = haystack.as_bytes();
        let p = prefix.as_bytes();
        if h.len() >= p.len() && h[..p.len()].eq_ignore_ascii_case(p) {
            Some(&haystack[p.len()..])
        } else {
            None
        }
    }
}
