use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_private_method() {
    let content = r#"class Foo {
   [|[|{| "contextRangeIndex": 0 |}#foo|]() { }|]
   callFoo() {
       return this.[|#foo|]();
   }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, ToAny(f.GetRangesByText().Get("#foo"))...)
}
