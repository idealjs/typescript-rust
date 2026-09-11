use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("documentHighlightYield", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, f.Ranges()[0])
}
