use super::*;

#[test]
fn test_validate_package_name_ok() {
    let (result, _, _) = validate_package_name("react");
    assert_eq!(result, NameValidationResult::NameOk);
}

#[test]
fn test_validate_package_name_empty() {
    let (result, _, _) = validate_package_name("");
    assert_eq!(result, NameValidationResult::EmptyName);
}

#[test]
fn test_validate_package_name_starts_with_dot() {
    let (result, _, _) = validate_package_name(".foo");
    assert_eq!(result, NameValidationResult::NameStartsWithDot);
}

#[test]
fn test_validate_package_name_starts_with_underscore() {
    let (result, _, _) = validate_package_name("_foo");
    assert_eq!(result, NameValidationResult::NameStartsWithUnderscore);
}

#[test]
fn test_validate_package_name_scoped() {
    let (result, _, _) = validate_package_name("@scope/package");
    assert_eq!(result, NameValidationResult::NameOk);
}
