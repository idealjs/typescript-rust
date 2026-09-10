use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_local_02() {
    let content = r#"const x = { [|hello|]: () => {} };

x.he/*function_call*/llo();
"#;
    let mut s = Session::new_for_test("goToImplementationLocal_02", content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "function_call")
}
