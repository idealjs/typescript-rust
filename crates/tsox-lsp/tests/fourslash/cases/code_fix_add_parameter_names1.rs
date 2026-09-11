use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_add_parameter_names1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noImplicitAny: true
var x: ([|number |]) => string;"#;
    let mut s = Session::new_for_test("codeFixAddParameterNames1", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `arg0: number`, false, 0, 0)
}
