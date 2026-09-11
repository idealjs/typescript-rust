use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_bad_import() {
    let content = r#"import { /*0*/ab as /*1*/cd } from "doesNotExist";"#;
    let mut s = Session::new_for_test("findAllRefsBadImport", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "0", "1")
}
