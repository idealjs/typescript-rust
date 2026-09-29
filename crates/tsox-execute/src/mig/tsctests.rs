#![allow(dead_code, unused_imports)]

use std::collections::HashMap;

use super::m5b::FileMap;

pub struct TscInput {
    pub sub_scenario: String,
    pub command_line_args: Option<Vec<String>>,
    pub files: FileMap,
    pub cwd: String,
    pub edits: Vec<TscEdit>,
    pub env: HashMap<String, String>,
    pub output_is_tty: Option<bool>,
    pub ignore_case: bool,
    pub windows_style_root: String,
}

impl Default for TscInput {
    fn default() -> Self {
        TscInput {
            sub_scenario: String::new(),
            command_line_args: None,
            files: FileMap::new(),
            cwd: String::new(),
            edits: Vec::new(),
            env: HashMap::new(),
            output_is_tty: None,
            ignore_case: false,
            windows_style_root: String::new(),
        }
    }
}

pub struct TscEdit {
    pub caption: String,
    pub command_line_args: Option<Vec<String>>,
    pub edit: Option<Box<dyn Fn(&super::m5b::TestSys)>>,
    pub expected_diff: String,
}

pub mod baseline {
    pub struct BaselineOptions {
        pub subfolder: String,
    }

    impl BaselineOptions {
        pub fn new(subfolder: &str) -> Self {
            BaselineOptions {
                subfolder: subfolder.to_string(),
            }
        }
    }

    pub fn local_root() -> std::path::PathBuf {
        std::path::PathBuf::from("tests/baselines/tsctests/local")
    }

    pub fn reference_root() -> std::path::PathBuf {
        std::path::PathBuf::from("tests/baselines/tsctests/reference")
    }

    pub fn run(file_name: &str, actual: &str, opts: &BaselineOptions) -> Result<(), String> {
        let local_path = local_root().join(&opts.subfolder).join(file_name);
        if let Some(parent) = local_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&local_path, actual).map_err(|e| e.to_string())
    }

    pub fn diff_text(old_name: &str, new_name: &str, expected: &str, actual: &str) -> String {
        fn split_lines(text: &str) -> Vec<&str> {
            if text.is_empty() {
                Vec::new()
            } else {
                text.split('\n')
                    .map(|l| l.trim_end_matches('\r'))
                    .collect()
            }
        }
        let a = split_lines(expected);
        let b = split_lines(actual);
        let n = a.len();
        let m = b.len();
        let mut lcs = vec![vec![0usize; m + 1]; n + 1];
        for i in (0..n).rev() {
            for j in (0..m).rev() {
                lcs[i][j] = if a[i] == b[j] {
                    lcs[i + 1][j + 1] + 1
                } else {
                    lcs[i + 1][j].max(lcs[i][j + 1])
                };
            }
        }
        #[derive(Clone, Copy, PartialEq)]
        enum Op {
            Same(usize, usize),
            Del(usize),
            Add(usize),
        }
        let mut ops: Vec<Op> = Vec::new();
        let (mut i, mut j) = (0usize, 0usize);
        while i < n && j < m {
            if a[i] == b[j] {
                ops.push(Op::Same(i, j));
                i += 1;
                j += 1;
            } else if lcs[i + 1][j] >= lcs[i][j + 1] {
                ops.push(Op::Del(i));
                i += 1;
            } else {
                ops.push(Op::Add(j));
                j += 1;
            }
        }
        while i < n {
            ops.push(Op::Del(i));
            i += 1;
        }
        while j < m {
            ops.push(Op::Add(j));
            j += 1;
        }

        const CONTEXT: usize = 3;
        let is_change = |op: &Op| !matches!(op, Op::Same(_, _));
        if !ops.iter().any(is_change) {
            return String::new();
        }

        let mut included = vec![false; ops.len()];
        for (idx, op) in ops.iter().enumerate() {
            if is_change(op) {
                let lo = idx.saturating_sub(CONTEXT);
                let hi = (idx + CONTEXT).min(ops.len() - 1);
                for slot in included.iter_mut().take(hi + 1).skip(lo) {
                    *slot = true;
                }
            }
        }

        let mut out = String::new();
        out.push_str(&format!("--- {old_name}\n"));
        out.push_str(&format!("+++ {new_name}\n"));
        let mut line_a = 1usize;
        let mut line_b = 1usize;
        let mut idx = 0usize;
        while idx < ops.len() {
            if !included[idx] {
                match ops[idx] {
                    Op::Same(_, _) => {
                        line_a += 1;
                        line_b += 1;
                    }
                    Op::Del(_) => line_a += 1,
                    Op::Add(_) => line_b += 1,
                }
                idx += 1;
                continue;
            }
            let start = idx;
            while idx < ops.len() && included[idx] {
                idx += 1;
            }
            let end = idx;
            let (mut a_count, mut b_count) = (0usize, 0usize);
            let mut body = String::new();
            for op in &ops[start..end] {
                match *op {
                    Op::Same(x, _) => {
                        a_count += 1;
                        b_count += 1;
                        body.push_str(&format!(" {}\n", a[x]));
                    }
                    Op::Del(x) => {
                        a_count += 1;
                        body.push_str(&format!("-{}\n", a[x]));
                    }
                    Op::Add(y) => {
                        b_count += 1;
                        body.push_str(&format!("+{}\n", b[y]));
                    }
                }
            }
            out.push_str(&format!(
                "@@ -{},{} +{},{} @@\n",
                line_a, a_count, line_b, b_count
            ));
            out.push_str(&body);
            for op in &ops[start..end] {
                match *op {
                    Op::Same(_, _) => {
                        line_a += 1;
                        line_b += 1;
                    }
                    Op::Del(_) => line_a += 1,
                    Op::Add(_) => line_b += 1,
                }
            }
        }
        out
    }
}
