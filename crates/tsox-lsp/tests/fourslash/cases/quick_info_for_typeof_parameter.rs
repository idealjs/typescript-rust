use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_for_typeof_parameter() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"function foo() {
    var y/*ref1*/1: string;
    var x: typeof y/*ref2*/1;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "ref1", "(local var) y1: string", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "ref2", "(local var) y1: string", "")
}
