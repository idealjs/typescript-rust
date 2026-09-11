use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_conflict_marker1() {
    let content = r#"class C {
<<<<<<< HEAD
v = 1;
=======
v = 2;
>>>>>>> Branch - a
}"#;
    let mut s = Session::new_for_test("formatConflictMarker1", content);
    // TODO: f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"class C {
<<<<<<< HEAD
v = 1;
=======
v = 2;
>>>>>>> Branch - a
}"#);
}
