use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: /*2*/                at projectPath : ${projectFile}"]
#[test]
fn formatting_multiline_template_literals() {
    let content = r#"/*1*/new Error(` + "`" + "#;
    // TODO: /*2*/                at projectPath : ${projectFile}
    // TODO: /*3*/                with error: ${ex.message}` + "`" + `)`
    let mut s = Session::new(content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifyCurrentLineContent"); // f.VerifyCurrentLineContent(t, "new Error(`Failed to expand glob: ${projectSpec.filesGlob}")
}
