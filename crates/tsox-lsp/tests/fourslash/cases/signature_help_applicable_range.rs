use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: // Markers a, b, c should NOT show signature help (outside t"]
#[test]
fn signature_help_applicable_range() {
    let content = r#"let obj = {
    foo(s: string): string {
        return s;
    }
};

let s =/*a*/ obj.foo("Hello, world!")/*b*/  
  /*c*/;"#;
    let mut s = Session::new_for_test("signatureHelpApplicableRange", content);
    // TODO: // Markers a, b, c should NOT show signature help (outside the call)
    fourslash::unsupported("VerifyNoSignatureHelpForMarkers"); // f.VerifyNoSignatureHelpForMarkers(t, "a", "b", "c")
}
