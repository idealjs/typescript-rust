use tsox_lsp::fourslash::{self, Session};


#[test]
fn document_highlights_33722() {
    let content = r#"// @Filename: /y.ts
class Foo {
  private foo() {}
}

const f = () => new Foo();
export default f;
// @Filename: /x.ts
import y from "./y";

y().[|foo|]();"#;
    let mut s = Session::new_for_test("documentHighlights_33722", content);
    // TODO: f.VerifyBaselineDocumentHighlightsWithOptions(t, nil /*preferences*/, []string{"/x.ts"}, f.Ranges()[
}
