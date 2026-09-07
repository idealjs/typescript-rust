use std::fmt;

use serde::{Deserialize, Serialize};

use crate::tspath;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DocumentUri(pub String);

impl DocumentUri {
    pub fn file_name(&self) -> String {
        let uri = &self.0;

        if crate::bundled::is_bundled(uri) {
            return uri.clone();
        }

        if let Some(rest) = uri.strip_prefix("file://") {
            if let Some(stripped) = rest.strip_prefix("//") {
                if let Some(slash_idx) = stripped.find('/') {
                    let (_host, path) = stripped.split_at(slash_idx);
                    return path.to_string();
                }
                return stripped.to_string();
            }

            if let Some(rest2) = rest.strip_prefix('/') {
                if rest2.len() >= 2 && rest2.as_bytes()[1] == b':' {
                    return rest2.to_string();
                }
            }
            return rest.to_string();
        }

        let (scheme, path, ok) = split_once(uri, ':');
        if !ok {
            panic!("invalid URI: {uri}");
        }

        let authority = "ts-nul-authority";
        let mut file_path = path;
        if let Some(rest) = path.strip_prefix("//") {
            let (_auth, rest_path, ok) = split_once(rest, '/');
            if ok {
                file_path = rest_path;
            }
        }

        format!("^/{scheme}/{authority}/{file_path}")
    }

    pub fn path(&self, use_case_sensitive_file_names: bool) -> tspath::Path {
        let file_name = self.file_name();
        tspath::to_path(&file_name, "", use_case_sensitive_file_names)
    }
}

impl fmt::Display for DocumentUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

pub type Uri = String;

pub type Method = String;

fn split_once(s: &str, sep: char) -> (&str, &str, bool) {
    match s.find(sep) {
        Some(idx) => (&s[..idx], &s[idx + sep.len_utf8()..], true),
        None => (s, "", false),
    }
}
