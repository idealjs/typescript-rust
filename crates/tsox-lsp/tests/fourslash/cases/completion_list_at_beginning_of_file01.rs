use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_at_beginning_of_file01() {
    let content = r#"/*1*/
var x = 0, y = 1, z = 2;
enum E {
    A, B, C
}"#;
    let mut s = Session::new_for_test("completionListAtBeginningOfFile01", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("1"), &["x", "y", "z", "E"], &[]);
}
