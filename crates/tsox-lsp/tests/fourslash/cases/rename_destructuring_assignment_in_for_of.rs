use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_destructuring_assignment_in_for_of() {
    let content = r#"// @strict: false
interface I {
    [|[|{| "contextRangeIndex": 0 |}property1|]: number;|]
    property2: string;
}
var elems: I[];

var [|[|{| "contextRangeIndex": 2 |}property1|]: number|], p2: number;
for ([|{ [|{| "contextRangeIndex": 4 |}property1|] } of elems|]) {
    [|property1|]++;
}
for ([|{ [|{| "contextRangeIndex": 7 |}property1|]: p2 } of elems|]) {
}"#;
    let mut s = Session::new_for_test("renameDestructuringAssignmentInForOf", content);
    fourslash::verify_no_errors(&mut s, );
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[8], f.Ranges()[3], f.Ranges
}
