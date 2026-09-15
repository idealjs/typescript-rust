use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_try_catch_finally4() {
    let content = r#"try/*1*/ {
    try/*2*/ {
    }
    catch/*3*/ (x) {
    }

    try/*4*/ {
    }
    finally/*5*/ {/*8*/
    }
}
catch/*6*/ (e) {
}
finally/*7*/ {
}"#;
    let _s = Session::new_for_test("getOccurrencesTryCatchFinally4", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Markers())...)
}
