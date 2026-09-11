use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_occurrences_async_await() {
    let content = r#"[|async|] function f() {
 [|await|] 100;
 [|a/**/wait|] [|await|] 200;
class Foo {
    async memberFunction() {
        await 1;
    }
}
 return [|await|] async function () {
   await 300;
 }
}
async function g() {
    await 300;
    async function f() {
        await 400;
    }
}"#;
    let mut s = Session::new_for_test("getOccurrencesAsyncAwait", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
