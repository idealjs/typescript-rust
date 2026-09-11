use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_parameter_property_declaration3() {
    let content = r#"class Foo {
    constructor([|protected [|{| "contextRangeIndex": 0 |}protectedParam|]: number|]) {
        let protectedParam = [|protectedParam|];
        this.[|protectedParam|] += 10;
    }
}"#;
    let mut s = Session::new_for_test("renameParameterPropertyDeclaration3", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "protectedParam")
}
