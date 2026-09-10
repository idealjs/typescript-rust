use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
#[test]
fn quickinfo_verbosity_empty_enum() {
    let content = r#"
enum Degree {}

declare const e/*0*/: Degree;
"#;
    let mut s = Session::new_for_test("quickinfoVerbosityEmptyEnum", content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"0": {0, 1}})
}
