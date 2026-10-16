#![allow(dead_code, unused_imports, unused_variables)]

use std::io::Write;

use super::m5b::{
    TestSys, BUILD_FINISHED_IN, BUILD_STARTING_AT, BUILD_STATUS_REPORT_END,
    BUILD_STATUS_REPORT_START, FAKE_DURATION, FAKE_TIME_STAMP, LIST_FILE_END, LIST_FILE_START,
    STATISTICS_END, STATISTICS_START, TRACE_END, TRACE_START, WATCH_STATUS_REPORT_END,
    WATCH_STATUS_REPORT_START,
};
use super::m5a2_2::{fs_baseline_util, harness_util};
use tsox_tsoptions::vfs::FS;

pub struct OutputSanitizer<'a> {
    pub for_comparing: bool,
    pub lines: Vec<&'a str>,
    pub index: usize,
    pub output_lines: Vec<String>,
}

impl<'a> OutputSanitizer<'a> {
    pub fn add_output_line(&mut self, s: &str) { ::tsox_core::fntrace::enter("add_output_line"); 
        let s = s.replace(
            &format!("'{}'", tsox_core::core::mig::m3k::version()),
            &format!("'{}'", harness_util::FAKE_TS_VERSION),
        );
        let s = s.replace(&english_version(), &fake_english_version());
        let s = s.replace(&czech_version(), &fake_czech_version());
        let s = fs_baseline_util::sanitize_internal_symbol_name(&s);
        self.output_lines.push(s);
    }

    pub fn sanitize_build_status_time_stamp(&self) -> String { ::tsox_core::fntrace::enter("sanitize_build_status_time_stamp"); 
        let status_line = self.lines[self.index];
        let hh_separator = status_line.find(':').expect("Expected timestamp");
        assert!(hh_separator >= 2, "Expected timestamp");
        format!(
            "{}{}{}",
            &status_line[..hh_separator - 2],
            FAKE_TIME_STAMP,
            &status_line[hh_separator - 2 + FAKE_TIME_STAMP.len()..]
        )
    }

    pub fn transform_lines(&mut self) -> String { ::tsox_core::fntrace::enter("transform_lines"); 
        while self.index < self.lines.len() {
            let line = self.lines[self.index];
            if line.starts_with(BUILD_STARTING_AT) {
                if !self.for_comparing {
                    self.add_output_line(&format!("{}{}", BUILD_STARTING_AT, FAKE_TIME_STAMP));
                }
                self.index += 1;
                continue;
            }
            if line.starts_with(BUILD_FINISHED_IN) {
                if !self.for_comparing {
                    self.add_output_line(&format!("{}{}", BUILD_FINISHED_IN, FAKE_DURATION));
                }
                self.index += 1;
                continue;
            }
            if !self.add_or_skip_lines_for_comparing(LIST_FILE_START, LIST_FILE_END, false, None)
                && !self.add_or_skip_lines_for_comparing(STATISTICS_START, STATISTICS_END, true, None)
                && !self.add_or_skip_lines_for_comparing(TRACE_START, TRACE_END, false, None)
                && !self.add_or_skip_lines_for_comparing(
                    BUILD_STATUS_REPORT_START,
                    BUILD_STATUS_REPORT_END,
                    false,
                    Some(&|s: &Self| s.sanitize_build_status_time_stamp()),
                )
                && !self.add_or_skip_lines_for_comparing(
                    WATCH_STATUS_REPORT_START,
                    WATCH_STATUS_REPORT_END,
                    false,
                    Some(&|s: &Self| s.sanitize_build_status_time_stamp()),
                )
            {
                self.add_output_line(line);
            }
            self.index += 1;
        }
        self.output_lines.join("\n")
    }

    pub fn add_or_skip_lines_for_comparing(
        &mut self,
        line_start: &str,
        line_end: &str,
        skip_even_if_not_comparing: bool,
        sanitize_first_line: Option<&dyn Fn(&Self) -> String>,
    ) -> bool { ::tsox_core::fntrace::enter("add_or_skip_lines_for_comparing"); 
        if self.lines[self.index] != line_start {
            return false;
        }
        self.index += 1;
        let mut is_first_line = true;
        while self.index < self.lines.len() {
            if self.lines[self.index] == line_end {
                return true;
            }
            if !self.for_comparing && !skip_even_if_not_comparing {
                let mut line = self.lines[self.index].to_string();
                if is_first_line {
                    if let Some(sanitize) = sanitize_first_line {
                        line = sanitize(self);
                    }
                    is_first_line = false;
                }
                self.add_output_line(&line);
            }
            self.index += 1;
        }
        panic!("Expected lineEnd{} not found after {}", line_end, line_start)
    }
}

fn english_version() -> String { ::tsox_core::fntrace::enter("english_version"); 
    tsox_core::diagnostics::VERSION_0.localize(&tsox_core::locale::Locale::default(), &[
        tsox_core::core::mig::m3k::version(),
    ])
}

fn fake_english_version() -> String { ::tsox_core::fntrace::enter("fake_english_version"); 
    tsox_core::diagnostics::VERSION_0.localize(&tsox_core::locale::Locale::default(), &[
        harness_util::FAKE_TS_VERSION,
    ])
}

fn czech_version() -> String { ::tsox_core::fntrace::enter("czech_version"); 
    tsox_core::diagnostics::VERSION_0.localize(&czech_locale(), &[
        tsox_core::core::mig::m3k::version(),
    ])
}

fn fake_czech_version() -> String { ::tsox_core::fntrace::enter("fake_czech_version"); 
    tsox_core::diagnostics::VERSION_0.localize(&czech_locale(), &[
        harness_util::FAKE_TS_VERSION,
    ])
}

fn czech_locale() -> tsox_core::locale::Locale { ::tsox_core::fntrace::enter("czech_locale"); 
    tsox_core::locale::Locale::parse("cs").expect("valid cs locale")
}

impl TestSys {
    pub fn write_file_no_error(&self, path: &str, content: &str) { ::tsox_core::fntrace::enter("write_file_no_error"); 
        self.fs_from_file_map()
            .write_file(path, content)
            .unwrap_or_else(|e| panic!("{}", e));
    }

    pub fn remove_no_error(&self, path: &str) { ::tsox_core::fntrace::enter("remove_no_error"); 
        self.fs_from_file_map()
            .remove(path)
            .unwrap_or_else(|e| panic!("{}", e));
    }

    pub fn read_file_no_error(&self, path: &str) -> String { ::tsox_core::fntrace::enter("read_file_no_error"); 
        self.fs_from_file_map()
            .read_file(path)
            .unwrap_or_else(|| panic!("File not found: {}", path))
    }

    pub fn rename_file_no_error(&self, old_path: &str, new_path: &str) { ::tsox_core::fntrace::enter("rename_file_no_error"); 
        let content = self.read_file_no_error(old_path);
        self.write_file_no_error(new_path, &content);
        self.remove_no_error(old_path);
    }

    pub fn replace_file_text(&self, path: &str, old_text: &str, new_text: &str) { ::tsox_core::fntrace::enter("replace_file_text"); 
        let content = self.read_file_no_error(path);
        let content = content.replacen(old_text, new_text, 1);
        self.write_file_no_error(path, &content);
    }

    pub fn replace_file_text_all(&self, path: &str, old_text: &str, new_text: &str) { ::tsox_core::fntrace::enter("replace_file_text_all"); 
        let content = self.read_file_no_error(path);
        let content = content.replace(old_text, new_text);
        self.write_file_no_error(path, &content);
    }

    pub fn append_file(&self, path: &str, text: &str) { ::tsox_core::fntrace::enter("append_file"); 
        let content = self.read_file_no_error(path);
        self.write_file_no_error(path, &format!("{}{}", content, text));
    }

    pub fn prepend_file(&self, path: &str, text: &str) { ::tsox_core::fntrace::enter("prepend_file"); 
        let content = self.read_file_no_error(path);
        self.write_file_no_error(path, &format!("{}{}", text, content));
    }
}
