use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRenameAtRangesWithText"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "propC")
}
