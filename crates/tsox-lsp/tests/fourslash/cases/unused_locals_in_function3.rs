use tsox_lsp::fourslash::{self, Session};


#[test]
fn unused_locals_in_function3() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noUnusedLocals: true
function greeter() {
   [| var x, y = 0,z = 1; |]
    x+1;
    z+1;
}"#;
    let mut s = Session::new_for_test("unusedLocalsInFunction3", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `var x,z = 1;`, false, 6133, 0)
}
