use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("getOccurrencesYield", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
