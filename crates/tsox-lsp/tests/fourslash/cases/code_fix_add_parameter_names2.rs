use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_add_parameter_names2() {
    let content = r#"// @noImplicitAny: true
type Rest = ([|...number|]) => void;"#;
    let _s = Session::new_for_test("codeFixAddParameterNames2", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `...arg0: number[]`, false, 0, 0)
}
