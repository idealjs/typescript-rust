use tsox_lsp::fourslash::Session;


#[test]
fn go_to_implementation_local_08() {
    let content = r#"declare function [|someFunction|](): () => void;
someFun/*reference*/ction();"#;
    let _s = Session::new_for_test("goToImplementationLocal_08", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "reference")
}
