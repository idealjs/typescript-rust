#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameValidationResult {
    NameOk,
    EmptyName,
    NameTooLong,
    NameStartsWithDot,
    NameStartsWithUnderscore,
    NameContainsNonUriSafeCharacters,
}

impl Default for NameValidationResult {
    fn default() -> Self {
        NameValidationResult::NameOk
    }
}

const MAX_PACKAGE_NAME_LENGTH: usize = 214;

pub fn validate_package_name(package_name: &str) -> (NameValidationResult, String, bool) {
    validate_package_name_worker(package_name, true)
}

fn validate_package_name_worker(
    package_name: &str,
    support_scoped_package: bool,
) -> (NameValidationResult, String, bool) {
    let package_name_len = package_name.len();
    if package_name_len == 0 {
        return (NameValidationResult::EmptyName, String::new(), false);
    }
    if package_name_len > MAX_PACKAGE_NAME_LENGTH {
        return (NameValidationResult::NameTooLong, String::new(), false);
    }
    let first_char = package_name.chars().next().unwrap();
    if first_char == '.' {
        return (
            NameValidationResult::NameStartsWithDot,
            String::new(),
            false,
        );
    }
    if first_char == '_' {
        return (
            NameValidationResult::NameStartsWithUnderscore,
            String::new(),
            false,
        );
    }

    if support_scoped_package {
        if let Some(without_scope) = package_name.strip_prefix('@') {
            if let Some((scope, scoped_package_name)) = without_scope.split_once('/') {
                if !scope.is_empty()
                    && !scoped_package_name.is_empty()
                    && !scoped_package_name.contains('/')
                {
                    let (scope_result, _, _) = validate_package_name_worker(scope, false);
                    if scope_result != NameValidationResult::NameOk {
                        return (scope_result, scope.to_string(), true);
                    }
                    let (package_result, _, _) =
                        validate_package_name_worker(scoped_package_name, false);
                    if package_result != NameValidationResult::NameOk {
                        return (package_result, scoped_package_name.to_string(), false);
                    }
                    return (NameValidationResult::NameOk, String::new(), false);
                }
            }
        }
    }

    if query_escape(package_name) != package_name {
        return (
            NameValidationResult::NameContainsNonUriSafeCharacters,
            String::new(),
            false,
        );
    }
    (NameValidationResult::NameOk, String::new(), false)
}

pub fn render_package_name_validation_failure(
    typing: &str,
    result: NameValidationResult,
    name: &str,
    is_scope_name: bool,
) -> String {
    let kind = if is_scope_name { "Scope" } else { "Package" };
    let name = if name.is_empty() { typing } else { name };
    match result {
        NameValidationResult::EmptyName => {
            format!("'{}':: {} name '{}' cannot be empty", typing, kind, name)
        }
        NameValidationResult::NameTooLong => format!(
            "'{}':: {} name '{}' should be less than {} characters",
            typing, kind, name, MAX_PACKAGE_NAME_LENGTH
        ),
        NameValidationResult::NameStartsWithDot => {
            format!(
                "'{}':: {} name '{}' cannot start with '.'",
                typing, kind, name
            )
        }
        NameValidationResult::NameStartsWithUnderscore => format!(
            "'{}':: {} name '{}' cannot start with '_'",
            typing, kind, name
        ),
        NameValidationResult::NameContainsNonUriSafeCharacters => format!(
            "'{}':: {} name '{}' contains non URI safe characters",
            typing, kind, name
        ),
        NameValidationResult::NameOk => panic!("Unexpected Ok result"),
    }
}

fn query_escape(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~') {
            result.push(c);
        } else {
            let mut buf = [0u8; 4];
            for &byte in c.encode_utf8(&mut buf).as_bytes() {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

#[cfg(test)]
mod tests;
