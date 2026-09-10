use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_local_06() {
    let content = r#"declare var [|someVar|]: string;
someVa/*reference*/r"#;
    let mut s = Session::new_for_test("goToImplementationLocal_06", content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "reference")
}
