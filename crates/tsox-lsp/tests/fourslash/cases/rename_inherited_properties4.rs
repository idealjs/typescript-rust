use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_inherited_properties4() {
    let content = r#"interface interface1 extends interface1 {
   [|[|{| "contextRangeIndex": 0 |}doStuff|](): string;|]
}

var v: interface1;
v.[|doStuff|]();"#;
    let mut s = Session::new_for_test("renameInheritedProperties4", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "doStuff")
}
