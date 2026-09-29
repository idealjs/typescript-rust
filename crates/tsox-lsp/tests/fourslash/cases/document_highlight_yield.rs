use tsox_lsp::fourslash::Session;


#[test]
fn document_highlight_yield() {
    let content = r#"
// @Filename: /a.ts
class C {
  async *[Symbol.asyncIterator]() {
    [|yield|] {
		type: 'type',
	};
  }
}
"#;
    let _s = Session::new_for_test("documentHighlightYield", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, f.Ranges()[0])
}
