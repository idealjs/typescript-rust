use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_no_space_after_template_head_and_middle() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"const a1 = ` + "`" + `${ 1 }${ 1 }` + "`" + `;
const a2 = ` + "`" + "#;
    // TODO: ${ 1 }${ 1 }
    // TODO: ` + "`" + `;
    // TODO: ${ 1 }${ 1 }
    // TODO: ` + "`" + `;
    // TODO: ${ 1 }${ 1 }
    // TODO: ` + "`" + `;
    // TODO: text ${ 1 }
    // TODO: text ${ 1 }
    // TODO: text
    // TODO: ` + "`" + `;`
    let mut s = Session::new_for_test("formatNoSpaceAfterTemplateHeadAndMiddle", content);
    // TODO: opts429 := f.GetOptions()
    // TODO: opts429.FormatCodeSettings.InsertSpaceAfterOpeningAndBeforeClosingTemplateStringBraces = core.TSFals
    // TODO: f.Configure(t, opts429)
    // TODO: f.FormatDocument(t, "")
    // TODO: f.VerifyCurrentFileContent(t, "const a1 = `${1}${1}`;\n"+"const a2 = `\n"+`    ${1}${1}
}
