use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineSignatureHelp"]
#[test]
fn signature_help_on_import_defer() {
    let content = r#"let m = import.defer(/**/)"#;
    let mut s = Session::new_for_test("signatureHelpOnImportDefer", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyBaselineSignatureHelp"); // f.VerifyBaselineSignatureHelp(t)
}
