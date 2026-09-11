use tsox_lsp::fourslash::{self, Session};


#[test]
fn references_bloom_filters3() {
    let content = r#"// @Filename: declaration.ts
enum Test { /*1*/"/*2*/42" = 1 };
// @Filename: expression.ts
(Test[/*3*/42]);"#;
    let mut s = Session::new_for_test("referencesBloomFilters3", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
