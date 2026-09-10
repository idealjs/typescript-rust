use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
#[test]
fn quickinfo_verbosity_const_enum() {
    let content = r#"
const enum Direction/*1*/ {
    Up = "UP",
    Down = "DOWN",
    Left = "LEFT",
    Right = "RIGHT",
}

enum NumericEnum/*2*/ {
    A,
    B = 10,
    C,
}
"#;
    let mut s = Session::new_for_test("quickinfoVerbosityConstEnum", content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{
}
