use tsox_lsp::fourslash::{self, Session};


#[test]
fn consistence_on_indentions_of_objects_in_a_list_after_formatting() {
    let content = r#"foo({
}, {/*1*/
});/*2*/"#;
    let mut s = Session::new_for_test("consistenceOnIndentionsOfObjectsInAListAfterFormatting", content);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCurrentLineContent(t, `
}
