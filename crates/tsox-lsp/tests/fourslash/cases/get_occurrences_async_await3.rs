use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_occurrences_async_await3() {
    let content = r#"a/**/wait 100;
async function f() {
    await 300;
}"#;
    let mut s = Session::new_for_test("getOccurrencesAsyncAwait3", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "")
}
