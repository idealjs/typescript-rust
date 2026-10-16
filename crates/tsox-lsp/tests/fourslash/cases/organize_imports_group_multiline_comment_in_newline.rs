use tsox_lsp::fourslash::Session;


#[test]
fn organize_imports_group_multiline_comment_in_newline() {
    let content = r#"// polyfill
import c from "C";
/*
* demo
*/
import d from "D";
import a from "A";
import b from "B";

console.log(a, b, c, d)"#;
    let _s = Session::new_for_test("organizeImportsGroup_MultilineCommentInNewline", content);
    // TODO: f.VerifyOrganizeImports(t,
}
