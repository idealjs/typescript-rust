use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_implementation_invalid() {
    let content = r#"var x1 = 50/*0*/0;
var x2 = "hel/*1*/lo";
/*2*/"#;
    let mut s = Session::new_for_test("goToImplementationInvalid", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "0", "1", "2")
}
