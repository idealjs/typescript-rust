use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRenameAtRangesWithText"]
#[test]
fn rename_inherited_properties8() {
    let content = r#"class C implements D {
    [|[|{| "contextRangeIndex": 0 |}prop1|]: string;|]
}

interface D extends C {
    [|[|{| "contextRangeIndex": 2 |}prop1|]: string;|]
}

var c: C;
c.[|prop1|];"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "prop1")
}
