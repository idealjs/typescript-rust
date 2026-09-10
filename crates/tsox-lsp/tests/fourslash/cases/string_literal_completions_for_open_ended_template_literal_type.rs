use tsox_lsp::fourslash::{self, Session};


#[test]
fn string_literal_completions_for_open_ended_template_literal_type() {
    let content = r#"// @stableTypeOrdering: true
function conversionTest(groupName: | "downcast" | "dataDowncast" | "editingDowncast" | ` + "`" + `${string}Downcast` + "`" + ` & {}) {}
conversionTest("/**/");"#;
    let mut s = Session::new_for_test("stringLiteralCompletionsForOpenEndedTemplateLiteralType", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["dataDowncast", "downcast", "editingDowncast"]);
}
