use tsox_lsp::fourslash::Session;


#[test]
fn go_to_implementation_local_01() {
    let content = r#"const [|hello|] = function() {};
he/*function_call*/llo();"#;
    let _s = Session::new_for_test("goToImplementationLocal_01", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "function_call")
}
