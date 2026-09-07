use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_infer_from_usage_member3() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noImplicitAny: true
class C {
    constructor([|public p)|] { }
}
new C("string");"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `public p: string)`, false, 0, 0)
}
