use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_import_equals() {
    let content = r#"import j = N./**/q;
namespace N { export const q = 0; }"#;
    let mut s = Session::new_for_test("findAllRefsImportEquals", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}
