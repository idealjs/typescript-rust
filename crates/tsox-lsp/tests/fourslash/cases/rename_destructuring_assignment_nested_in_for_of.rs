use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_destructuring_assignment_nested_in_for_of() {
    let content = r#"// @strict: false
interface MultiRobot {
    name: string;
    skills: {
        [|[|{| "contextRangeIndex": 0 |}primary|]: string;|]
        secondary: string;
    };
}
let multiRobots: MultiRobot[];
let [|[|{| "contextRangeIndex": 2 |}primary|]: string|], secondary: string, primaryA: string, secondaryA: string;
for ([|{ skills: { [|{| "contextRangeIndex": 4 |}primary|]: primaryA, secondary: secondaryA } } of multiRobots|]) {
    primaryA;
}
for ([|{ skills: { [|{| "contextRangeIndex": 6 |}primary|], secondary } } of multiRobots|]) {
    [|primary|];
}"#;
    let mut s = Session::new_for_test("renameDestructuringAssignmentNestedInForOf", content);
    fourslash::verify_no_errors(&mut s, );
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[5], f.Ranges()[3], f.Ranges
}
