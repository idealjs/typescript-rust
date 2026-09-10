use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_this_00() {
    let content = r#"class [|Bar|] extends Foo {
    hello() {
        thi/*this_call*/s.whatever();
    }

    whatever() {}
}"#;
    let mut s = Session::new_for_test("goToImplementationThis_00", content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "this_call")
}
