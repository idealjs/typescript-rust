use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_destructuring_assignment_nested_in_for_of2() {
    let content = r#"interface MultiRobot {
    name: string;
    skills: {
        [|[|{| "contextRangeIndex": 0 |}primary|]: string;|]
        secondary: string;
    };
}
let multiRobots: MultiRobot[], [|[|{| "contextRangeIndex": 2 |}primary|]: string|];
for ([|{ skills: { [|{| "contextRangeIndex": 4 |}primary|]: primaryA, secondary: secondaryA } } of multiRobots|]) {
    console.log(primaryA);
}
for ([|{ skills: { [|{| "contextRangeIndex": 6 |}primary|], secondary } } of multiRobots|]) {
    console.log([|primary|]);
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[5], f.Ranges()[3], f.Ranges
}
