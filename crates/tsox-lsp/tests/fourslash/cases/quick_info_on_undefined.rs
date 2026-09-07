use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_on_undefined() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"function foo(a: string) {
}
foo(/*1*/undefined);
var x = {
    undefined: 10
};
x./*2*/undefined = 30;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "var undefined", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(property) undefined: number", "")
}
