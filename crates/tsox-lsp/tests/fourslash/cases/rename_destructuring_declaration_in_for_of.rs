use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_destructuring_declaration_in_for_of() {
    let content = r#"interface I {
    [|[|{| "contextRangeIndex": 0 |}property1|]: number;|]
    property2: string;
}
var elems: I[];

for ([|let { [|{| "contextRangeIndex": 2 |}property1|] } of elems|]) {
    [|property1|]++;
}
for ([|let { [|{| "contextRangeIndex": 5 |}property1|]: p2 } of elems|]) {
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[6], f.Ranges()[3], f.Ranges
}
