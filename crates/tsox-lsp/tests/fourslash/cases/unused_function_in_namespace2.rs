use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_function_in_namespace2() {
    let content = r#"// @noUnusedLocals: true
 [| namespace greeter {
    export function function2() {
    }
    function function1() {
    }
} |]"#;
    let mut s = Session::new_for_test("unusedFunctionInNamespace2", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `namespace greeter {
}
