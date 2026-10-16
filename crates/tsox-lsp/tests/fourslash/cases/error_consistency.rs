use tsox_lsp::fourslash::{self, Session};


#[test]
fn error_consistency() {
    let content = r#"interface Int<T> {
val<U>(f: (t: T) => U): Int<U>;
}
declare var v1: Int<string>;
var /*1*/v2/*2*/: Int<number> = v1;"#;
    let mut s = Session::new_for_test("errorConsistency", content);
    fourslash::go_to_eof(&mut s, );
    // TODO: f.VerifyErrorExistsBetweenMarkers(t, "1", "2")
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
    // TODO: f.Backspace(t, 1)
    // TODO: f.VerifyErrorExistsBetweenMarkers(t, "1", "2")
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
}
