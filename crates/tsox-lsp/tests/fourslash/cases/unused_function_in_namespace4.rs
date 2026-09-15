use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_function_in_namespace4() {
    let content = r#"// @noUnusedLocals: true
// @noUnusedParameters:true
 [| namespace Validation {
    var function1 = function() {
    }
} |]"#;
    let _s = Session::new_for_test("unusedFunctionInNamespace4", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `namespace Validation {
}
