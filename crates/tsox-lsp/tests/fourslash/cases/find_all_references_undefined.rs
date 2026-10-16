use tsox_lsp::fourslash::Session;


#[test]
fn find_all_references_undefined() {
    let content = r#"// @Filename: /a.ts
/**/undefined;

void undefined;
// @Filename: /b.ts
undefined;"#;
    let _s = Session::new_for_test("findAllReferencesUndefined", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}
