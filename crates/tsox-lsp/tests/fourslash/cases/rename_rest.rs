use tsox_lsp::fourslash::Session;


#[test]
fn rename_rest() {
    let content = r#"interface Gen {
    x: number;
    [|[|{| "contextRangeIndex": 0 |}parent|]: Gen;|]
    millenial: string;
}
let t: Gen;
var { x, ...rest } = t;
rest.[|parent|];"#;
    let _s = Session::new_for_test("renameRest", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "parent")
}
