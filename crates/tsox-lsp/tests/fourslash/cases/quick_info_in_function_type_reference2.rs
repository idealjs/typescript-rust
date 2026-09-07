use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_in_function_type_reference2() {
    let content = r#"class C<T> {
    map(fn: (/*1*/k: string, /*2*/value: T, context: any) => void, context: any) {
    }
}
var c: C<number>;
c.map(/*3*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(parameter) k: string", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(parameter) value: T", "")
    fourslash::go_to_marker(&mut s, "3");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "map(fn: (k: string, value: numb
}
