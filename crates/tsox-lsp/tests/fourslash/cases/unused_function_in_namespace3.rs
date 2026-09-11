use tsox_lsp::fourslash::{self, Session};


#[test]
fn unused_function_in_namespace3() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noUnusedLocals: true
// @noUnusedParameters:true
 [| namespace Validation {
    function function1() {
    }
} |]"#;
    let mut s = Session::new_for_test("unusedFunctionInNamespace3", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `namespace Validation {
}
