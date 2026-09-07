use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_namespace_02() {
    let content = r#"namespace Foo {
    export function [|hello|]() {}
}

Foo.hell/*reference*/o();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "reference")
}
