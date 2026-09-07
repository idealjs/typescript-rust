use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyErrorExistsBetweenMarkers"]
#[test]
fn squiggle_illegal_class_extension() {
    let content = r#"class Foo extends /*1*/Bar/*2*/ { }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyErrorExistsBetweenMarkers"); // f.VerifyErrorExistsBetweenMarkers(t, "1", "2")
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
}
