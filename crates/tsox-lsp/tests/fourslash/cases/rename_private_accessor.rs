use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_private_accessor() {
    let content = r#"class Foo {
   [|get [|{| "contextRangeIndex": 0 |}#foo|]() { return 1 }|]
   [|set [|{| "contextRangeIndex": 2 |}#foo|](value: number) { }|]
   retFoo() {
       return this.[|#foo|];
   }
}"#;
    let mut s = Session::new_for_test("renamePrivateAccessor", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, ToAny(f.GetRangesByText().Get("#foo"))...)
}
