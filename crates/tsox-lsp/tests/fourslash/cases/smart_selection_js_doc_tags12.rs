use tsox_lsp::fourslash::{self, Session};


#[test]
fn smart_selection_js_doc_tags12() {
    let content = r#"type B = {};
type A = {
    a(/** Comment */ /*1*/p0: number, /** Comment */ /*2*/p1: number, /** Comment */ /*3*/p2: number): string;
};"#;
    let mut s = Session::new_for_test("smartSelection_JSDocTags12", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}
