use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineSelectionRanges"]
#[test]
fn smart_selection_function_params1() {
    let content = r#"function f(/*1*/p, /*2*/q?, /*3*/...r: any[] = []) {}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineSelectionRanges"); // f.VerifyBaselineSelectionRanges(t)
}
