use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_on_module_indentation() {
    let content = r#"  namespace     Foo    {
    export    namespace    A  .   B  .   C     {      }/**/
               }"#;
    let mut s = Session::new_for_test("formattingOnModuleIndentation", content);
    fourslash::format_document(&mut s, "");
    // TODO: f.GoToBOF(t)
    fourslash::verify_current_line_content(&mut s, r#"namespace Foo {"#);
    fourslash::go_to_marker(&mut s, "");
    fourslash::verify_current_line_content(&mut s, r#"    export namespace A.B.C { }"#);
    // TODO: f.GoToEOF(t)
    fourslash::verify_current_line_content(&mut s, r#"}"#);
}
