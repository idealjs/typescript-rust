use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_export_not_at_top_level() {
    let content = r#"{
    /*1*/export const /*2*/x = 0;
    /*3*/x;
}"#;
    let mut s = Session::new_for_test("findAllRefsExportNotAtTopLevel", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
