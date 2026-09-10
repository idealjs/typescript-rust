use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: ${1}${1}"]
#[test]
fn format_space_after_template_head_and_middle() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"const a1 = ` + "`" + `${1}${1}` + "`" + `;
const a2 = ` + "`" + "#;
    // TODO: ${1}${1}
    // TODO: ` + "`" + `;
    // TODO: ${1}${1}
    // TODO: ` + "`" + `;
    // TODO: ${1}${1}
    // TODO: ` + "`" + `;
    // TODO: text ${1}
    // TODO: text ${1}
    // TODO: text
    // TODO: ` + "`" + `;`
    let mut s = Session::new_for_test("formatSpaceAfterTemplateHeadAndMiddle", content);
    // TODO: opts405 := f.GetOptions()
    // TODO: opts405.FormatCodeSettings.InsertSpaceAfterOpeningAndBeforeClosingTemplateStringBraces = core.TSTrue
    fourslash::unsupported("Configure"); // f.Configure(t, opts405)
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::unsupported("VerifyCurrentFileContent"); // f.VerifyCurrentFileContent(t, "const a1 = `${ 1 }${ 1 }`;\n"+"const a2 = `\n"+`    ${ 1 }${ 1 }
}
