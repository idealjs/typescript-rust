use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_invalid() {
    let content = r#"var x1 = 50/*0*/0;
var x2 = "hel/*1*/lo";
/*2*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "0", "1", "2")
}
