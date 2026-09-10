use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn formatting_on_module_indentation() {
    let content = r#"  namespace     Foo    {
    export    namespace    A  .   B  .   C     {      }/**/
               }"#;
    let mut s = Session::new_for_test("formattingOnModuleIndentation", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::unsupported("GoToBOF"); // f.GoToBOF(t)
    fourslash::verify_current_line_content(&mut s, r#"namespace Foo {"#);
    fourslash::go_to_marker(&mut s, "");
    fourslash::verify_current_line_content(&mut s, r#"    export namespace A.B.C { }"#);
    fourslash::unsupported("GoToEOF"); // f.GoToEOF(t)
    fourslash::verify_current_line_content(&mut s, r#"}"#);
}
