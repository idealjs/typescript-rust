use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_occurrences_return3() {
    let content = r#"function f(a: number) {
    if (a > 0) {
        return (function () {
            return;
            return;
            return;

            if (false) {
                return true;
            }
        })() || true;
    }

    var unusued = [1, 2, 3, 4].map(x => { [|return|] 4 })

    return;
    return true;
}"#;
    let mut s = Session::new_for_test("getOccurrencesReturn3", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
