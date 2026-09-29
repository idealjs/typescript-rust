use tsox_lsp::fourslash::Session;


#[test]
fn go_to_implementation_local_00() {
    let content = r#"he/*function_call*/llo();
function [|hello|]() {}"#;
    let _s = Session::new_for_test("goToImplementationLocal_00", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "function_call")
}
