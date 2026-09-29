use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_re_export_broken() {
    let content = r#"// @Filename: /a.ts
/*1*/export { /*2*/x };"#;
    let _s = Session::new_for_test("findAllRefsReExport_broken", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}
