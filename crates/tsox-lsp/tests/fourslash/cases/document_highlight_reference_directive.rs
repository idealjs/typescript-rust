use tsox_lsp::fourslash::Session;


#[test]
fn document_highlight_reference_directive() {
    let content = r#"// @Filename: /a.ts
/// <reference path="[|./b.ts|]" />

const x = 1;

// @filename: b.ts
export type Foo = number;
"#;
    let _s = Session::new_for_test("documentHighlightReferenceDirective", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, f.Ranges()[0])
}
