use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn references_bloom_filters3() {
    let content = r#"// @Filename: declaration.ts
enum Test { /*1*/"/*2*/42" = 1 };
// @Filename: expression.ts
(Test[/*3*/42]);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
