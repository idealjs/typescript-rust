use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.DeleteAtCaret"]
#[test]
fn unused_label_after_edit() {
    let content = r#"// @allowUnusedLabels: false
myLabel: while (true) {
    if (Math.random() > 0.5) {
        /*marker*/break myLabel;
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 0)
    fourslash::go_to_marker(&mut s, "marker");
    fourslash::unsupported("DeleteAtCaret"); // f.DeleteAtCaret(t, 14)
    fourslash::insert(&mut s, "break;");
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 1)
    fourslash::go_to_marker(&mut s, "marker");
    fourslash::unsupported("DeleteAtCaret"); // f.DeleteAtCaret(t, 6)
    fourslash::insert(&mut s, "break myLabel;");
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 0)
}
