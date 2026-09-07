use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn document_highlight_reference_directive() {
    let content = r#"// @Filename: /a.ts
/// <reference path="[|./b.ts|]" />

const x = 1;

// @filename: b.ts
export type Foo = number;
"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, f.Ranges()[0])
}
