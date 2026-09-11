use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_inherited_properties3() {
    let content = r#"interface interface1 extends interface1 {
   [|[|{| "contextRangeIndex": 0 |}propName|]: string;|]
}

var v: interface1;
v.[|propName|];"#;
    let mut s = Session::new_for_test("renameInheritedProperties3", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "propName")
}
