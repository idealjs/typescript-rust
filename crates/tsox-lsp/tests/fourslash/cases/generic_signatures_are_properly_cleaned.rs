use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.DeleteAtCaret"]
#[test]
fn generic_signatures_are_properly_cleaned() {
    let content = r#"interface Int<T> {
val<U>(f: (t: T) => U): Int<U>;
}
declare var v1: Int<string>;
var v2: Int<number> = v1/*1*/;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 1)
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("DeleteAtCaret"); // f.DeleteAtCaret(t, 1)
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 1)
}
