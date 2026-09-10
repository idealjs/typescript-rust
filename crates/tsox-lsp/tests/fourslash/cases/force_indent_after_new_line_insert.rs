use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn force_indent_after_new_line_insert() {
    let content = r#"function f1()
{ return 0; }
function f2()
{
return 0;
}
function g()
{ function h() {
return 0;
}}"#;
    let mut s = Session::new_for_test("forceIndentAfterNewLineInsert", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"function f1() { return 0; }
function f2() {
    return 0;
}
function g() {
    function h() {
        return 0;
    }
}"#);
}
