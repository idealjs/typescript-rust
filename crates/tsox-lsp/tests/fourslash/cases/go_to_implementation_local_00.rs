use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_local_00() {
    let content = r#"he/*function_call*/llo();
function [|hello|]() {}"#;
    let mut s = Session::new_for_test("goToImplementationLocal_00", content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "function_call")
}
