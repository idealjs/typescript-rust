use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_parameter_property_declaration4() {
    let content = r#"class Foo {
    constructor([|protected { [|{| "contextRangeIndex": 0 |}protectedParam|] }|]) {
        let myProtectedParam = [|protectedParam|];
    }
}"#;
    let mut s = Session::new_for_test("renameParameterPropertyDeclaration4", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[2])
}
