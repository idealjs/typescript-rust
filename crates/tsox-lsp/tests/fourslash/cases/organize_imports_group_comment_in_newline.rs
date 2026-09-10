use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyOrganizeImports"]
#[test]
fn organize_imports_group_comment_in_newline() {
    let content = r#"// polyfill
import c from "C";
// not polyfill
import d from "D";
import a from "A";
import b from "B";

console.log(a, b, c, d)"#;
    let mut s = Session::new_for_test("organizeImportsGroup_CommentInNewline", content);
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}
