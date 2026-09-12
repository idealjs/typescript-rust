use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_multiline_template_literals() {
    let content = r#"/*1*/new Error(`Failed to expand glob: ${projectSpec.filesGlob}
/*2*/                at projectPath : ${projectFile}
/*3*/                with error: ${ex.message}`)"#;
    let mut s = Session::new_for_test("formattingMultilineTemplateLiterals", content);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCurrentLineContent(t, "new Error(`Failed to expand glob: ${projectSpec.filesGlob}")
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"                at projectPath : ${projectFile}"#);
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifyCurrentLineContent(t, "                with error: ${ex.message}`)")
}
