use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_implementation_local_07() {
    let content = r#"declare function [|someFunction|](): () => void;
someFun/*reference*/ction();"#;
    let mut s = Session::new_for_test("goToImplementationLocal_07", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "reference")
}
