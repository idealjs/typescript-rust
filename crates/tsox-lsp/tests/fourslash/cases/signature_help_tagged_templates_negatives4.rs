use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_tagged_templates_negatives4() {
    let content = r#"function foo(strs, ...rest) {
}

/*1*/fo/*2*/o /*3*/``/*4*/  /*5*/"#;
    let mut s = Session::new_for_test("signatureHelpTaggedTemplatesNegatives4", content);
    // TODO: f.VerifyNoSignatureHelpForMarkers(t, f.MarkerNames()...)
}
