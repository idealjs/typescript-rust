use tsox_lsp::fourslash::Session;


#[test]
fn go_to_implementation_local_03() {
    let content = r#"let [|he/*local_var*/llo|] = {};

x.hello();

hello = {};
"#;
    let _s = Session::new_for_test("goToImplementationLocal_03", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "local_var")
}
