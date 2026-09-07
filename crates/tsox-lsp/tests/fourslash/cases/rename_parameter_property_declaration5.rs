use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRenameAtRangesWithText"]
#[test]
fn rename_parameter_property_declaration5() {
    let content = r#"class Foo {
    constructor([|protected [ [|{| "contextRangeIndex": 0 |}protectedParam|] ]|]) {
        let myProtectedParam = [|protectedParam|];
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "protectedParam")
}
