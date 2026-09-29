use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_re_export_broken2() {
    let content = r#"// @Filename: /a.ts
/*1*/export { /*2*/x } from "nonsense";"#;
    let _s = Session::new_for_test("findAllRefsReExport_broken2", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}
