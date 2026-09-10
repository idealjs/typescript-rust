use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_super_00() {
    let content = r#"class [|Foo|] {
    constructor() {}
}

class Bar extends Foo {
    constructor() {
        su/*super_call*/per();
    }
}"#;
    let mut s = Session::new_for_test("goToImplementationSuper_00", content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "super_call")
}
