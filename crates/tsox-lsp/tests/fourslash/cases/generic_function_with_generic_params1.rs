use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn generic_function_with_generic_params1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"var obj = function f<T>(a: T) {
    var x/**/x: T;
    return a;
};"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "(local var) xx: T", "")
}
