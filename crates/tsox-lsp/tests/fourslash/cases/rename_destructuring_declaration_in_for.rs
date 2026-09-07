use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_destructuring_declaration_in_for() {
    let content = r#"interface I {
    [|[|{| "contextRangeIndex": 0 |}property1|]: number;|]
    property2: string;
}
var elems: I[];

var p2: number, property1: number;
for ([|let { [|{| "contextRangeIndex": 2 |}property1|]: p2 } = elems[0]|]; p2 < 100; p2++) {
}
for ([|let { [|{| "contextRangeIndex": 4 |}property1|] } = elems[0]|]; p2 < 100; p2++) {
    [|property1|] = p2;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[3], f.Ranges()[5], f.Ranges
}
