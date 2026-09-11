use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_inherited_properties7() {
    let content = r#"class C extends D {
    [|[|{| "contextRangeIndex": 0 |}prop1|]: string;|]
}

class D extends C {
    prop1: string;
}

var c: C;
c.[|prop1|];"#;
    let mut s = Session::new_for_test("renameInheritedProperties7", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "prop1")
}
