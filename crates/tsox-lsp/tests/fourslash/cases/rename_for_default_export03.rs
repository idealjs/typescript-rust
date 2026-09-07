use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_for_default_export03() {
    let content = r#"[|function /*1*/[|{| "contextRangeIndex": 0 |}f|]() {
    return 100;
}|]

[|export default /*2*/[|{| "contextRangeIndex": 2 |}f|];|]

var x: typeof /*3*/[|f|];

var y = /*4*/[|f|]();

/**
 *  Commenting [|{| "inComment": true |}f|]
 */
[|namespace /*5*/[|{| "contextRangeIndex": 7 |}f|] {
    var local = 100;
}|]"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, ToAny(core.Filter(f.GetRangesByText().Get("f"), func(
}
