use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_const04() {
    let content = r#"export const class C {
    private static c/*1*/onst f/*2*/oo;
    constructor(public con/*3*/st foo) {
    }
}"#;
    let _s = Session::new_for_test("getOccurrencesConst04", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "1", "2", "3")
}
