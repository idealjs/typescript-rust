use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_selection_after_template_literal1() {
    let content = r#"const a = `head${"x"};
`;

/*begin*/export const f = () => {
    return `world`;
/*end*/}
"#;
    let mut s = Session::new_for_test("formatSelectionAfterTemplateLiteral1", content);
    fourslash::format_selection(&mut s, "begin", "end");
    // TODO: f.VerifyCurrentFileContent(t, "const a = `head${\"x\"};\n`;\n\nexport const f = () => {\n    return 
}
