use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_infer_from_usage_variable3_js() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @noEmit: true
// @noImplicitAny: false
// @Filename: important.js
[|function f(foo) {
    foo += 2
    return foo
}|]"#;
    let mut s = Session::new_for_test("codeFixInferFromUsageVariable3JS", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `/** 
}
