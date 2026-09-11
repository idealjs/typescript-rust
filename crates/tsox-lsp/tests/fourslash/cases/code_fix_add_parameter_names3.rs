use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_add_parameter_names3() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noImplicitAny: true
type Rest = ([|public string|]) => void;"#;
    let mut s = Session::new_for_test("codeFixAddParameterNames3", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `public arg0: string`, false, 0, 0)
}
