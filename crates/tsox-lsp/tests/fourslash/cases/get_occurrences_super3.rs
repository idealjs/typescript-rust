use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_super3() {
    let content = r#"let x = {
    a() {
        return [|s/**/uper|].b();
    },
    b() {
        return [|super|].a();
    },
    c: function () {
        return [|super|].a();
    }
    d: () => [|super|].b();
}"#;
    let _s = Session::new_for_test("getOccurrencesSuper3", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
