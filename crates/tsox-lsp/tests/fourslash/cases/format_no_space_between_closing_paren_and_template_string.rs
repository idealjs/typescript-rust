use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_no_space_between_closing_paren_and_template_string() {
    let content = r#"foo() ` + "`" + `abc` + "`" + `;
bar()` + "`" + `def` + "`" + `;
baz()` + "`" + `a${x}b` + "`" + `;"#;
    let mut s = Session::new_for_test("formatNoSpaceBetweenClosingParenAndTemplateString", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::unsupported("VerifyCurrentFileContent"); // f.VerifyCurrentFileContent(t, "foo()`abc`;\nbar()`def`;\nbaz()`a${x}b`;")
}
