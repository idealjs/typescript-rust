use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.VerifyCurrentLineContent(t, `"]
#[test]
fn consistence_on_indentions_of_objects_in_a_list_after_formatting() {
    let content = r#"foo({
}, {/*1*/
});/*2*/"#;
    let mut s = Session::new_for_test("consistenceOnIndentionsOfObjectsInAListAfterFormatting", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCurrentLineContent(t, `
}
