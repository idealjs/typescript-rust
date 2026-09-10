use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
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
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, f.Ranges()[0])
}
