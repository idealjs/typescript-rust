use tsox_lsp::fourslash::Session;


#[test]
fn rename_parameter_property_declaration5() {
    let content = r#"class Foo {
    constructor([|protected [ [|{| "contextRangeIndex": 0 |}protectedParam|] ]|]) {
        let myProtectedParam = [|protectedParam|];
    }
}"#;
    let _s = Session::new_for_test("renameParameterPropertyDeclaration5", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "protectedParam")
}
