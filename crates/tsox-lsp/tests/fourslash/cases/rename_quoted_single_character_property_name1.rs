use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_quoted_single_character_property_name1() {
    let content = r#""#;
    // TODO: "\n" +
    // TODO: "const obj = {\n" +
    let mut s = Session::new_for_test("renameQuotedSingleCharacterPropertyName1", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
