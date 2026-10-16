use tsox_lsp::fourslash::Session;


#[test]
fn document_highlight_in_keyword() {
    let content = r#"export type Foo<T> = {
    [K [|in|] keyof T]: any;
}

"a" [|in|] {};

for (let a [|in|] {}) {}"#;
    let _s = Session::new_for_test("documentHighlightInKeyword", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
