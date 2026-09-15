use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_export_not_at_top_level() {
    let content = r#"{
    /*1*/export const /*2*/x = 0;
    /*3*/x;
}"#;
    let _s = Session::new_for_test("findAllRefsExportNotAtTopLevel", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
