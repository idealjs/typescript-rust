use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn js_signature_41059() {
    let content = r#"// @lib: esnext
// @allowNonTsExtensions: true
// @Filename: Foo.js
a.next(/**/);"#;
    let mut s = Session::new_for_test("jsSignature_41059", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "Generator.next(): IteratorResul
}
