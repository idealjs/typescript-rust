use super::*;

#[test]
fn tristate_basics() {
    assert!(Tristate::True.is_true());
    assert!(Tristate::False.is_false());
    assert!(Tristate::Unknown.is_unknown());
    assert!(Tristate::True.is_true_or_unknown());
    assert!(Tristate::Unknown.is_true_or_unknown());
    assert!(!Tristate::False.is_true_or_unknown());
}

#[test]
fn default_if_unknown() {
    assert_eq!(
        Tristate::Unknown.default_if_unknown(Tristate::True),
        Tristate::True
    );
    assert_eq!(
        Tristate::False.default_if_unknown(Tristate::True),
        Tristate::False
    );
}
