use tsox_lsp::fourslash::{self, Session};


#[test]
fn document_highlight_template_strings() {
    let content = r#"type Foo = "[|a|]" | "b";

class C {
   p: Foo = `[|a|]`;
   m() {
       switch (this.p) {
           case `[|a|]`:
               return 1;
           case "b":
               return 2;
       }
   }
}"#;
    let mut s = Session::new_for_test("documentHighlightTemplateStrings", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, f.Ranges()[2])
}
