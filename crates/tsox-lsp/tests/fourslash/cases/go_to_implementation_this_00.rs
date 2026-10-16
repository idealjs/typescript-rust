use tsox_lsp::fourslash::Session;


#[test]
fn go_to_implementation_this_00() {
    let content = r#"class [|Bar|] extends Foo {
    hello() {
        thi/*this_call*/s.whatever();
    }

    whatever() {}
}"#;
    let _s = Session::new_for_test("goToImplementationThis_00", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "this_call")
}
