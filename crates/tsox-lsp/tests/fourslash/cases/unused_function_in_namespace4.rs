use tsox_lsp::fourslash::{self, Session};


#[test]
fn unused_function_in_namespace4() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noUnusedLocals: true
// @noUnusedParameters:true
 [| namespace Validation {
    var function1 = function() {
    }
} |]"#;
    let mut s = Session::new_for_test("unusedFunctionInNamespace4", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `namespace Validation {
}
