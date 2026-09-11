use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_parameter_property_declaration2() {
    let content = r#"class Foo {
    constructor([|public [|{| "contextRangeIndex": 0 |}publicParam|]: number|]) {
        let publicParam = [|publicParam|];
        this.[|publicParam|] += 10;
    }
}"#;
    let mut s = Session::new_for_test("renameParameterPropertyDeclaration2", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "publicParam")
}
