use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_private_method() {
    let content = r#"class Foo {
   [|[|{| "contextRangeIndex": 0 |}#foo|]() { }|]
   callFoo() {
       return this.[|#foo|]();
   }
}"#;
    let mut s = Session::new_for_test("renamePrivateMethod", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, ToAny(f.GetRangesByText().Get("#foo"))...)
}
