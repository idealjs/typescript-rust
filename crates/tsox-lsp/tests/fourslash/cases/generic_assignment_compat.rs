use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyErrorExistsBetweenMarkers"]
#[test]
fn generic_assignment_compat() {
    let content = r#"interface Int<T> {

    val<U>(f: (t: T) => U): Int<U>;

}

declare var v1: Int<string>;

var /*1*/v2/*2*/: Int<number> = v1;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyErrorExistsBetweenMarkers"); // f.VerifyErrorExistsBetweenMarkers(t, "1", "2")
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 1)
}
