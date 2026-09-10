use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_this_01() {
    let content = r#"class [|Bar|] extends Foo {
    hello(): th/*this_type*/is {
        return this;
    }
}"#;
    let mut s = Session::new_for_test("goToImplementationThis_01", content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "this_type")
}
