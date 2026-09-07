use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn get_occurrences_yield() {
    let content = r#"function* f() {
 [|yield|] 100;
 [|y/**/ield|] [|yield|] 200;
  class Foo {
      *memberFunction() {
          return yield 1;
      }
  }
  return function* g() {
    yield 1;
  }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
