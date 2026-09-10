use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_merging_grandchildren() {
    let content = r#"// Should not merge grandchildren with property assignments
const o = {
    a: {
        m() {},
    },
    b: {
        m() {},
    },
}"#;
    let mut s = Session::new_for_test("navigationBarMerging_grandchildren", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
