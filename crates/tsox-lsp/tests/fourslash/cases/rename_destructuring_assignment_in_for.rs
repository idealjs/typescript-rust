use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_destructuring_assignment_in_for() {
    let content = r#"// @strict: false
interface I {
    [|[|{| "contextRangeIndex": 0 |}property1|]: number;|]
    property2: string;
}
var elems: I[];

var p2: number, [|[|{| "contextRangeIndex": 2 |}property1|]: number|];
for ([|{ [|{| "contextRangeIndex": 4 |}property1|] } = elems[0]|]; p2 < 100; p2++) {
   p2 = [|property1|]++;
}
for ([|{ [|{| "contextRangeIndex": 7 |}property1|]: p2 } = elems[0]|]; p2 < 100; p2++) {
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[8], f.Ranges()[3], f.Ranges
}
