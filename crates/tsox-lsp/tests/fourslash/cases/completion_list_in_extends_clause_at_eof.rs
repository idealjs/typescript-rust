use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_extends_clause_at_eof() {
    let content = r#"declare namespace mod {
    class Foo { }
}
class Bar extends mod./**/"#;
    let mut s = Session::new_for_test("completionListInExtendsClauseAtEOF", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some(""), &["Foo"], &[]);
}
