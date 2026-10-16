use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn format_no_space_after_template_head_and_middle() {
    let content = r#"const a1 = `${ 1 }${ 1 }`;
const a2 = `
    ${ 1 }${ 1 }
`;
const a3 = `


    ${ 1 }${ 1 }
`;
const a4 = `

    ${ 1 }${ 1 }

`;
const a5 = `text ${ 1 } text ${ 1 } text`;
const a6 = `
    text ${ 1 }
    text ${ 1 }
    text
`;"#;
    let mut s = Session::new_for_test("formatNoSpaceAfterTemplateHeadAndMiddle", content);
    // TODO: opts429 := f.GetOptions()
    fourslash::configure_format_settings(&mut s, &[("insert_space_after_opening_and_before_closing_template_string_braces", "false")]);
    fourslash::format_document(&mut s, "");
    // TODO: f.VerifyCurrentFileContent(t, "const a1 = `${1}${1}`;\n"+"const a2 = `\n"+`    ${1}${1}
}
