use tsox_lsp::fourslash::Session;


#[test]
fn go_to_implementation_class_method_00() {
    let content = r#"class Bar {
    [|{|"parts": ["(","method",")"," ","Bar",".","hello","(",")",":"," ","void"], "kind": "method"|}hello|]() {}
}

new Bar().hel/*reference*/lo;"#;
    let _s = Session::new_for_test("goToImplementationClassMethod_00", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "reference")
}
