use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.DeleteAtCaret"]
#[test]
fn paste_lambda_over_module() {
    let content = r#"// @strict: false
/**/"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("Paste"); // f.Paste(t, "namespace B { }")
    fourslash::unsupported("GoToBOF"); // f.GoToBOF(t)
    fourslash::unsupported("DeleteAtCaret"); // f.DeleteAtCaret(t, 15)
    fourslash::insert(&mut s, "var t = (public x) => { };");
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
}
