use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn contextual_typing_of_generic_call_signatures2() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"interface I {
    <T>(x: T): void
}
function f6(x: <T extends I>(p: T) => void) { }
// x should not be contextually typed so this should be an error
f6(/**/x => x<number>())"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "(parameter) x: T extends I", "")
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 1)
}
