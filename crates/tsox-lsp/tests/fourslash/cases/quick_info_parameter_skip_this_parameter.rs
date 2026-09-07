use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_parameter_skip_this_parameter() {
    let content = r#"function f(cb: (x: number) => void) {}
f(function(this: any, /**/x) {});"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "(parameter) x: number", "")
}
