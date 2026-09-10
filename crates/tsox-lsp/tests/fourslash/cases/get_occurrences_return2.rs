use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn get_occurrences_return2() {
    let content = r#"function f(a: number) {
    if (a > 0) {
        return (function () {
            [|return|];
            [|ret/**/urn|];
            [|return|];

            while (false) {
                [|return|] true;
            }
        })() || true;
    }

    var unusued = [1, 2, 3, 4].map(x => { return 4 })

    return;
    return true;
}"#;
    let mut s = Session::new_for_test("getOccurrencesReturn2", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
