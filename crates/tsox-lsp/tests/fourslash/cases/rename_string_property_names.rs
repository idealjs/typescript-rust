use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRenameAtRangesWithText"]
#[test]
fn rename_string_property_names() {
    let content = r#"var o = {
    [|[|{| "contextRangeIndex": 0 |}prop|]: 0|]
};

o = {
    [|"[|{| "contextRangeIndex": 2 |}prop|]": 1|]
};

o["[|prop|]"];
o['[|prop|]'];
o.[|prop|];"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "prop")
}
