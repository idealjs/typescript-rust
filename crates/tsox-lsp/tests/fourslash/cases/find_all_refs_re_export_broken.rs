use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_re_export_broken() {
    let content = r#"// @Filename: /a.ts
/*1*/export { /*2*/x };"#;
    let mut s = Session::new_for_test("findAllRefsReExport_broken", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2")
}
