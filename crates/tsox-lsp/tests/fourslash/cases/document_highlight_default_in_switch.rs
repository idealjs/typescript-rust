use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn document_highlight_default_in_switch() {
    let content = r#"const foo = 'foo';
[|switch|] (foo) {
   [|case|] 'foo':
       [|break|];
   [|default|]:
       [|break|];
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[4])
}
