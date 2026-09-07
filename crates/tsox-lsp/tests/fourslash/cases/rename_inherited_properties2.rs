use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRenameAtRangesWithText"]
#[test]
fn rename_inherited_properties2() {
    let content = r#"class class1 extends class1 {
   [|[|{| "contextRangeIndex": 0 |}doStuff|]() { }|]
}

var v: class1;
v.[|doStuff|]();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "doStuff")
}
