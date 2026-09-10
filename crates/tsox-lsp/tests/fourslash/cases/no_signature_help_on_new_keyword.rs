use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyNoSignatureHelpForMarkers"]
#[test]
fn no_signature_help_on_new_keyword() {
    let content = r#"class Foo { }
new/*1*/ Foo
new /*2*/Foo(/*3*/)"#;
    let mut s = Session::new_for_test("noSignatureHelpOnNewKeyword", content);
    fourslash::unsupported("VerifyNoSignatureHelpForMarkers"); // f.VerifyNoSignatureHelpForMarkers(t, "1", "2")
    fourslash::go_to_marker(&mut s, "3");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "Foo(): Foo"})
}
