use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_implementation_namespace_02() {
    let content = r#"namespace Foo {
    export function [|hello|]() {}
}

Foo.hell/*reference*/o();"#;
    let mut s = Session::new_for_test("goToImplementationNamespace_02", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "reference")
}
