use tsox_lsp::fourslash::Session;


#[test]
fn go_to_implementation_local_02() {
    let content = r#"const x = { [|hello|]: () => {} };

x.he/*function_call*/llo();
"#;
    let _s = Session::new_for_test("goToImplementationLocal_02", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "function_call")
}
