use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_default_lib_dont_work() {
    let content = r#"// @Filename: file1.ts
[|var [|{| "contextRangeIndex": 0 |}test|] = "foo";|]
console.log([|test|]);"#;
    let mut s = Session::new_for_test("renameDefaultLibDontWork", content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1])
}
