use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_destructuring_nested_binding_element() {
    let content = r#"interface MultiRobot {
    name: string;
    skills: {
        [|[|{| "contextRangeIndex": 0|}primary|]: string;|]
        secondary: string;
    };
}
let multiRobots: MultiRobot[];
for ([|let { skills: {[|{| "contextRangeIndex": 2|}primary|]: primaryA, secondary: secondaryA } } of multiRobots|]) {
    console.log(primaryA);
}
for ([|let { skills: {[|{| "contextRangeIndex": 4|}primary|], secondary } } of multiRobots|]) {
    console.log([|primary|]);
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[3], f.Ranges()[5], f.Ranges
}
