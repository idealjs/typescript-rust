use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_add_parameter_names3() {
    let content = r#"// @noImplicitAny: true
type Rest = ([|public string|]) => void;"#;
    let _s = Session::new_for_test("codeFixAddParameterNames3", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `public arg0: string`, false, 0, 0)
}
