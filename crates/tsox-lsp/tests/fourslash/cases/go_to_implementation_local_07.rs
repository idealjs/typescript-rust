use tsox_lsp::fourslash::Session;


#[test]
fn go_to_implementation_local_07() {
    let content = r#"declare function [|someFunction|](): () => void;
someFun/*reference*/ction();"#;
    let _s = Session::new_for_test("goToImplementationLocal_07", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "reference")
}
