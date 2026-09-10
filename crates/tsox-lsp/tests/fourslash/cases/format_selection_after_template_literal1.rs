use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: const content = 'const a = `head${\'x\'};\n`;\n\n/*begin*/ex"]
#[test]
fn format_selection_after_template_literal1() {
    // TODO: const content = "const a = `head${\"x\"};\n`;\n\n/*begin*/export const f = () => {\n    return `worl
    let mut s = Session::new_for_test("formatSelectionAfterTemplateLiteral1", "");
    fourslash::unsupported("FormatSelection"); // f.FormatSelection(t, "begin", "end")
    fourslash::unsupported("VerifyCurrentFileContent"); // f.VerifyCurrentFileContent(t, "const a = `head${\"x\"};\n`;\n\nexport const f = () => {\n    return 
}
