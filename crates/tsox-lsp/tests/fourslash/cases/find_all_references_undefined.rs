use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_references_undefined() {
    let content = r#"// @Filename: /a.ts
/**/undefined;

void undefined;
// @Filename: /b.ts
undefined;"#;
    let mut s = Session::new_for_test("findAllReferencesUndefined", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "")
}
