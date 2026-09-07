use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNoErrors"]
#[test]
fn get_semantic_diagnostic_for_declaration1() {
    let content = r#"// @declaration: true
// @Filename: File.d.ts
declare var v: string;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
}
