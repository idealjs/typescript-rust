use super::*;

#[test]
fn test_remove_min_and_version_numbers() {
    assert_eq!(remove_min_and_version_numbers("jquery-min.4.2.3"), "jquery");
    assert_eq!(
        remove_min_and_version_numbers("angular-route.1.2.3"),
        "angular-route"
    );
    assert_eq!(remove_min_and_version_numbers("jquery"), "jquery");
    assert_eq!(remove_min_and_version_numbers("jquery.min"), "jquery");
}
