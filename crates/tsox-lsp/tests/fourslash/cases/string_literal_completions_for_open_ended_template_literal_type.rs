use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn string_literal_completions_for_open_ended_template_literal_type() {
    let content = r#"// @stableTypeOrdering: true
function conversionTest(groupName: | "downcast" | "dataDowncast" | "editingDowncast" | ` + "`" + `${string}Downcast` + "`" + ` & {}) {}
conversionTest("/**/");"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
