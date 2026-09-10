use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: '\n' +"]
#[test]
fn rename_quoted_single_character_property_name1() {
    // TODO: const content = "" +
    // TODO: "\n" +
    // TODO: "const obj = {\n" +
    let mut s = Session::new_for_test("renameQuotedSingleCharacterPropertyName1", "");
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
