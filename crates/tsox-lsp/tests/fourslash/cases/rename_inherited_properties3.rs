use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRenameAtRangesWithText"]
#[test]
fn rename_inherited_properties3() {
    let content = r#"interface interface1 extends interface1 {
   [|[|{| "contextRangeIndex": 0 |}propName|]: string;|]
}

var v: interface1;
v.[|propName|];"#;
    let mut s = Session::new_for_test("renameInheritedProperties3", content);
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "propName")
}
