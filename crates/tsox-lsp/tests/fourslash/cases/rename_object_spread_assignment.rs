use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_object_spread_assignment() {
    let content = r#"interface A1 { a: number };
interface A2 { a?: number };
[|let [|{| "contextRangeIndex": 0 |}a1|]: A1;|]
[|let [|{| "contextRangeIndex": 2 |}a2|]: A2;|]
let a12 = { ...[|a1|], ...[|a2|] };"#;
    let mut s = Session::new_for_test("renameObjectSpreadAssignment", content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[4], f.Ranges()[3], f.Ranges
}
