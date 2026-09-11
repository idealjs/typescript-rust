use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_occurrences_throw6() {
    let content = r#"[|throw|] 100;

try {
    throw 0;
    var x = () => { throw 0; };
}
catch (y) {
    var x = () => { throw 0; };
    [|throw|] 200;
}
finally {
    [|throw|] 300;
}"#;
    let mut s = Session::new_for_test("getOccurrencesThrow6", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
