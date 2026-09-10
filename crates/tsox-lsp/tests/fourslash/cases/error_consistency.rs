use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.Backspace"]
#[test]
fn error_consistency() {
    let content = r#"interface Int<T> {
val<U>(f: (t: T) => U): Int<U>;
}
declare var v1: Int<string>;
var /*1*/v2/*2*/: Int<number> = v1;"#;
    let mut s = Session::new_for_test("errorConsistency", content);
    fourslash::unsupported("GoToEOF"); // f.GoToEOF(t)
    fourslash::unsupported("VerifyErrorExistsBetweenMarkers"); // f.VerifyErrorExistsBetweenMarkers(t, "1", "2")
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
    fourslash::unsupported("Backspace"); // f.Backspace(t, 1)
    fourslash::unsupported("VerifyErrorExistsBetweenMarkers"); // f.VerifyErrorExistsBetweenMarkers(t, "1", "2")
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
}
