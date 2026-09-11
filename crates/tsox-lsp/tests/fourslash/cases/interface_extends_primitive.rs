use tsox_lsp::fourslash::{self, Session};


#[test]
fn interface_extends_primitive() {
    let content = r#"interface x extends /*1*/string/*2*/ { }"#;
    let mut s = Session::new_for_test("interfaceExtendsPrimitive", content);
    // TODO: f.VerifyErrorExistsBetweenMarkers(t, "1", "2")
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
}
