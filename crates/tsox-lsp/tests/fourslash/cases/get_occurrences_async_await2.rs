use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_occurrences_async_await2() {
    let content = r#"[|a/**/sync|] function f() {
 [|await|] 100;
 [|await|] [|await|] 200;
 return [|await|] async function () {
   await 300;
 }
}"#;
    let mut s = Session::new_for_test("getOccurrencesAsyncAwait2", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
