use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_add_parameter_names1() {
    let content = r#"// @noImplicitAny: true
var x: ([|number |]) => string;"#;
    let _s = Session::new_for_test("codeFixAddParameterNames1", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `arg0: number`, false, 0, 0)
}
