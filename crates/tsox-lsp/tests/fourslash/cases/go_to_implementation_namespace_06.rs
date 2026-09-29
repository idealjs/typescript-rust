use tsox_lsp::fourslash::Session;


#[test]
fn go_to_implementation_namespace_06() {
    let content = r#"namespace [|F/*declaration*/oo|] {
    declare function hello(): void;
}


let x: typeof Foo = [|{ hello() {} }|];"#;
    let _s = Session::new_for_test("goToImplementationNamespace_06", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "declaration")
}
