use tsox_lsp::fourslash::Session;


#[test]
fn rename_private_fields1() {
    let content = r#"class Foo {
   [|[|{| "contextRangeIndex": 0 |}#foo|] = 1;|]

   getFoo() {
       return this.[|#foo|];
   }
}"#;
    let _s = Session::new_for_test("renamePrivateFields1", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "#foo")
}
