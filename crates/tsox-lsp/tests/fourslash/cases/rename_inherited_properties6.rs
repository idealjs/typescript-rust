use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_inherited_properties6() {
    let content = r#"interface C extends D {
    propD: number;
}
interface D extends C {
    [|[|{| "contextRangeIndex": 0 |}propC|]: number;|]
}
var d: D;
d.[|propC|];"#;
    let mut s = Session::new_for_test("renameInheritedProperties6", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "propC")
}
