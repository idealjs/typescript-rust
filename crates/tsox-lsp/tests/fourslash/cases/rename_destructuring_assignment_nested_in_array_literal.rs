use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_destructuring_assignment_nested_in_array_literal() {
    let content = r#"interface I {
    [|[|{| "contextRangeIndex": 0 |}property1|]: number;|]
    property2: string;
}
var elems: I[], p1: number, [|[|{| "contextRangeIndex": 2 |}property1|]: number|];
[|[{ [|{| "contextRangeIndex": 4 |}property1|]: p1 }] = elems;|]
[|[{ [|{| "contextRangeIndex": 6 |}property1|] }] = elems;|]"#;
    let mut s = Session::new_for_test("renameDestructuringAssignmentNestedInArrayLiteral", content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[5], f.Ranges()[3], f.Ranges
}
