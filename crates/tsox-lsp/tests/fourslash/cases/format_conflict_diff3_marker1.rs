use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_conflict_diff3_marker1() {
    let content = r#"class C {
<<<<<<< HEAD
v = 1;
||||||| merged common ancestors
v = 3;
=======
v = 2;
>>>>>>> Branch - a
}"#;
    let mut s = Session::new_for_test("formatConflictDiff3Marker1", content);
    // TODO: f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"class C {
<<<<<<< HEAD
v = 1;
||||||| merged common ancestors
v = 3;
=======
v = 2;
>>>>>>> Branch - a
}"#);
}
