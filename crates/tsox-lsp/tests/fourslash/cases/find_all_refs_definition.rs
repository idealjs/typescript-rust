use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_definition() {
    let content = r#"const /*1*/x = 0;
/*2*/x;"#;
    let mut s = Session::new_for_test("findAllRefsDefinition", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}
