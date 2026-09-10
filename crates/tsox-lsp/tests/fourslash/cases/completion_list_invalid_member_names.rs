use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_list_invalid_member_names() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r##"var x = {
    "foo ": "space in the name",
    "bar": "valid identifier name",
    "break": "valid identifier name (matches a keyword)",
    "any": "valid identifier name (matches a typescript keyword)",
    "#": "invalid identifier name",
    "$": "valid identifier name",
    "\u0062": "valid unicode identifier name (b)",
    "\u0031\u0062": "invalid unicode identifier name (1b)"
};

x[|./*a*/|];
x["[|/*b*/|]"];"##;
    let mut s = Session::new_for_test("completionListInvalidMemberNames", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "b", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
}
