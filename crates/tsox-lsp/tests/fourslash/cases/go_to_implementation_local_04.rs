use tsox_lsp::fourslash::Session;


#[test]
fn go_to_implementation_local_04() {
    let content = r#"function [|he/*local_var*/llo|]() {}

hello();
"#;
    let _s = Session::new_for_test("goToImplementationLocal_04", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "local_var")
}
