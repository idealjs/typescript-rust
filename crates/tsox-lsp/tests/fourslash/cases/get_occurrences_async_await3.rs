use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn get_occurrences_async_await3() {
    let content = r#"a/**/wait 100;
async function f() {
    await 300;
}"#;
    let mut s = Session::new_for_test("getOccurrencesAsyncAwait3", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "")
}
