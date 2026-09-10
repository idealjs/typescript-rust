use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_for_string_literal_quote_preference3() {
    let content = r##"const a = {
    "#": "a"
};
a[|./**/|]"##;
    let mut s = Session::new_for_test("completionForStringLiteral_quotePreference3", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
