use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_quoted_single_character_property_name1() {
    let content = r#"
const obj = {
  "'"/**/: 1,
}
"#;
    let mut s = Session::new_for_test("renameQuotedSingleCharacterPropertyName1", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
