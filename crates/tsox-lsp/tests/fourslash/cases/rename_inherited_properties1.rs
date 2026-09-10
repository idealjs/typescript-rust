use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRenameAtRangesWithText"]
#[test]
fn rename_inherited_properties1() {
    let content = r#"class class1 extends class1 {
   [|[|{| "contextRangeIndex": 0 |}propName|]: string;|]
}

var v: class1;
v.[|propName|];"#;
    let mut s = Session::new_for_test("renameInheritedProperties1", content);
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "propName")
}
