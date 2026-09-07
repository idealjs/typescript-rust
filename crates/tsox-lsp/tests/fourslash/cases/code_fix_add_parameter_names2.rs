use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_add_parameter_names2() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noImplicitAny: true
type Rest = ([|...number|]) => void;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `...arg0: number[]`, false, 0, 0)
}
