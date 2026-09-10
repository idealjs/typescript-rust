use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_destructuring_assignment_nested_in_for() {
    let content = r#"interface MultiRobot {
    name: string;
    skills: {
        [|[|{| "contextRangeIndex": 0 |}primary|]: string;|]
        secondary: string;
    };
}
declare let multiRobot: MultiRobot, [|[|{| "contextRangeIndex": 2 |}primary|]: string|], secondary: string, primaryA: string, secondaryA: string, i: number;
for ([|{ skills: { [|{| "contextRangeIndex": 4 |}primary|]: primaryA, secondary: secondaryA } } = multiRobot|], i = 0; i < 1; i++) {
    primaryA;
}
for ([|{ skills: { [|{| "contextRangeIndex": 6 |}primary|], secondary } } = multiRobot|], i = 0; i < 1; i++) {
    [|primary|];
}"#;
    let mut s = Session::new_for_test("renameDestructuringAssignmentNestedInFor", content);
    fourslash::verify_no_errors(&mut s, );
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[5], f.Ranges()[3], f.Ranges
}
