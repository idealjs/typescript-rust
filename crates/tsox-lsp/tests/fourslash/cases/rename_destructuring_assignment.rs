use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRenameAtRangesWithText"]
#[test]
fn rename_destructuring_assignment() {
    let content = r#"interface I {
    [|[|{| "contextRangeIndex": 0 |}x|]: number;|]
}
var a: I;
var x;
([|{ [|{| "contextRangeIndex": 2 |}x|]: x } = a|]);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "x")
}
