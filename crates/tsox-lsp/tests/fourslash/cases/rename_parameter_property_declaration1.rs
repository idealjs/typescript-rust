use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_parameter_property_declaration1() {
    let content = r#"class Foo {
    constructor([|private [|{| "contextRangeIndex": 0 |}privateParam|]: number|]) {
        let localPrivate = [|privateParam|];
        this.[|privateParam|] += 10;
    }
}"#;
    let mut s = Session::new_for_test("renameParameterPropertyDeclaration1", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "privateParam")
}
