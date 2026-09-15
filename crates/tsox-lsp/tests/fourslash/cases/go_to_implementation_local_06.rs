use tsox_lsp::fourslash::Session;


#[test]
fn go_to_implementation_local_06() {
    let content = r#"declare var [|someVar|]: string;
someVa/*reference*/r"#;
    let _s = Session::new_for_test("goToImplementationLocal_06", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "reference")
}
