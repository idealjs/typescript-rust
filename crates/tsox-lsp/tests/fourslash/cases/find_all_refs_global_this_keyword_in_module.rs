use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_global_this_keyword_in_module() {
    let content = r#"// @noLib: true
/*1*/this;
export const c = 1;"#;
    let mut s = Session::new_for_test("findAllRefsGlobalThisKeywordInModule", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
