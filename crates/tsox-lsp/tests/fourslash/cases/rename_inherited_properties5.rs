use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_inherited_properties5() {
    let content = r#"interface C extends D {
    propC: number;
}
interface D extends C {
    [|[|{| "contextRangeIndex": 0 |}propD|]: string;|]
}
var d: D;
d.[|propD|];"#;
    let mut s = Session::new_for_test("renameInheritedProperties5", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "propD")
}
