use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_inherited_properties1() {
    let content = r#"class class1 extends class1 {
   [|[|{| "contextRangeIndex": 0 |}propName|]: string;|]
}

var v: class1;
v.[|propName|];"#;
    let mut s = Session::new_for_test("renameInheritedProperties1", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "propName")
}
