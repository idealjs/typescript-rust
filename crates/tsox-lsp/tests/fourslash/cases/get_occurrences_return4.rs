use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_occurrences_return4() {
    let content = r#"function f(a: number) {
    if (a > 0) {
        return (function () {
            return/*1*/;
            return/*2*/;
            return/*3*/;

            if (false) {
                return/*4*/ true;
            }
        })() || true;
    }

    var unusued = [1, 2, 3, 4].map(x => { return/*5*/ 4 })

    return/*6*/;
    return/*7*/ true;
}"#;
    let mut s = Session::new_for_test("getOccurrencesReturn4", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Markers())...)
}
