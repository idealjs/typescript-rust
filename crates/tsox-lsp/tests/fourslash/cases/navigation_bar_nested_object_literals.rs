use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_nested_object_literals() {
    let content = r#"var a = {
    b: 0,
    c: {},
    d: {
        e: 1,
    },
    f: {
        g: 2,
        h: {
            i: 3,
        },
    },
}"#;
    let mut s = Session::new_for_test("navigationBarNestedObjectLiterals", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
