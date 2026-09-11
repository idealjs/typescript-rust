use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_export_default_class_constructor() {
    let content = r#"export default class {
    /*1*/constructor() {}
}"#;
    let mut s = Session::new_for_test("findAllRefsExportDefaultClassConstructor", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
