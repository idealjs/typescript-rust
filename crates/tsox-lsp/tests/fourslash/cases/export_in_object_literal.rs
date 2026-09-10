use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlightsWithOptions"]
#[test]
fn export_in_object_literal() {
    let content = r#"// @Filename: a.ts
const k = {
    [|export|] f() { }
}"#;
    let mut s = Session::new_for_test("exportInObjectLiteral", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlightsWithOptions"); // f.VerifyBaselineDocumentHighlightsWithOptions(t, nil /*preferences*/, []string{f.Ranges()[0].FileNam
}
