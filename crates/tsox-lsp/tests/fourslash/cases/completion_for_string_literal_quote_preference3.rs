use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_for_string_literal_quote_preference3() {
    let content = r##"const a = {
    "#": "a"
};
a[|./**/|]"##;
    let mut s = Session::new_for_test("completionForStringLiteral_quotePreference3", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
