//! Test case parser — ported from typescript-go's `testrunner/test_case_parser.go`.
//!
//! Parses multi-file TypeScript test cases that use `// @FileName` directives
//! and `// @Option: value` metadata to configure compiler options per test.
//!
//! A typical test file looks like:
//! ```text
//! // @filename: file1.ts
//! const x: number = 42;
//!
//! // @filename: file2.ts
//! import { x } from "./file1";
//!
//! // @strict: true
//! // @target: esnext
//! ```

use std::collections::HashMap;

use regex::Regex;

/// A single virtual file parsed from a multi-file test case.
#[derive(Debug, Clone)]
pub struct TestUnit {
    pub name: String,
    pub content: String,
}

/// The parsed content of a test case: virtual files + compiler settings.
#[derive(Debug, Clone)]
pub struct TestCaseContent {
    pub units: Vec<TestUnit>,
    pub tsconfig_content: Option<String>,
    /// Raw compiler settings parsed from `// @Option: value` lines.
    pub settings: HashMap<String, String>,
    /// The `currentDirectory` from `// @currentDirectory: ...`.
    pub current_directory: String,
}

/// Parses a multi-file test case from source text.
///
/// Handles:
/// - `// @filename: path.ts` — starts a new virtual file
/// - `// @OptionName: value` — compiler option or test config
/// - `// @currentDirectory: /path` — sets the virtual cwd
/// - Everything else — appended to the current file's content
pub fn parse_test_files(content: &str, default_filename: &str) -> TestCaseContent {
    let option_re = Regex::new(r"(?m)^//\s*@(\w+)\s*:\s*([^\r\n]*)").unwrap();

    let mut units: Vec<TestUnit> = Vec::new();
    let mut settings: HashMap<String, String> = HashMap::new();
    let mut current_directory = String::new();

    let mut current_content = String::new();
    let mut current_name = String::new();
    let mut has_content = false;

    for line in content.lines() {
        if let Some(caps) = option_re.captures(line) {
            let name = caps[1].to_lowercase();
            let value = caps[2].trim().to_string();

            if name == "filename" {
                // Save the previous file if any.
                if !current_name.is_empty() && has_content {
                    units.push(TestUnit {
                        name: current_name.clone(),
                        content: current_content.clone(),
                    });
                }
                current_name = value.trim().to_string();
                current_content.clear();
                has_content = false;
            } else if name == "currentdirectory" {
                current_directory = value;
            } else {
                settings.insert(name, value);
            }
        } else {
            // Content line.
            if !current_name.is_empty() {
                if !current_content.is_empty() {
                    current_content.push('\n');
                }
                current_content.push_str(line);
                has_content = true;
            } else {
                // Content before first @filename — for single-file tests, use the
                // default filename.
                if !current_content.is_empty() {
                    current_content.push('\n');
                }
                current_content.push_str(line);
                current_name = default_filename.to_string();
                has_content = true;
            }
        }
    }

    // Save the last file.
    if has_content && !current_name.is_empty() {
        units.push(TestUnit {
            name: current_name.clone(),
            content: current_content,
        });
    }

    // Extract tsconfig.json if present.
    let tsconfig_content = units
        .iter()
        .find(|u| u.name.ends_with("tsconfig.json"))
        .map(|u| u.content.clone());
    if tsconfig_content.is_some() {
        units.retain(|u| !u.name.ends_with("tsconfig.json"));
    }

    // Normalize current directory.
    if current_directory.is_empty() {
        current_directory = "/.src".to_string();
    }

    TestCaseContent {
        units,
        tsconfig_content,
        settings,
        current_directory,
    }
}

/// Extract compiler settings from `// @Option: value` lines at the top of a file.
/// Returns a map of lowercase option name → raw value string.
pub fn extract_compiler_settings(content: &str) -> HashMap<String, String> {
    let option_re = Regex::new(r"(?m)^//\s*@(\w+)\s*:\s*([^\r\n]*)").unwrap();
    let mut settings = HashMap::new();
    for caps in option_re.captures_iter(content) {
        let name = caps[1].to_lowercase();
        let value = caps[2].trim().to_string();
        // Skip file-directive options that aren't compiler settings.
        if !matches!(name.as_str(), "filename" | "currentdirectory" | "symlink") {
            settings.insert(name, value);
        }
    }
    settings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_file() {
        let content = "const x: number = 42;\n";
        let result = parse_test_files(content, "test.ts");
        assert_eq!(result.units.len(), 1);
        assert_eq!(result.units[0].name, "test.ts");
        assert!(result.units[0].content.contains("const x"));
    }

    #[test]
    fn parse_multi_file() {
        let content = "\
// @filename: a.ts
export const x = 1;

// @filename: b.ts
import { x } from './a';
";
        let result = parse_test_files(content, "test.ts");
        assert_eq!(result.units.len(), 2);
        assert_eq!(result.units[0].name, "a.ts");
        assert_eq!(result.units[1].name, "b.ts");
    }

    #[test]
    fn parse_compiler_settings() {
        let content = "\
// @strict: true
// @target: esnext
const x = 1;
";
        let settings = extract_compiler_settings(content);
        assert_eq!(settings.get("strict"), Some(&"true".to_string()));
        assert_eq!(settings.get("target"), Some(&"esnext".to_string()));
    }

    #[test]
    fn parse_tsconfig_file() {
        let content = "\
// @filename: tsconfig.json
{ \"compilerOptions\": { \"strict\": true } }

// @filename: main.ts
const x = 1;
";
        let result = parse_test_files(content, "test.ts");
        assert!(result.tsconfig_content.is_some());
        assert_eq!(result.units.len(), 1);
        assert_eq!(result.units[0].name, "main.ts");
    }

    #[test]
    fn parse_current_directory() {
        let content = "\
// @currentDirectory: /project/src
const x = 1;
";
        let result = parse_test_files(content, "test.ts");
        assert_eq!(result.current_directory, "/project/src");
    }
}
