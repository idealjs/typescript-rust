use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_object_literal_assignment_pattern1() {
    let content = r#"let x = { a: 1, b: 2 };
let y = ({ /**/ } = x, 1);"#;
    let mut s = Session::new_for_test("completionListInObjectLiteralAssignmentPattern1", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["a", "b"]);
}
