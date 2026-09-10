use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_global_this_keyword_in_module() {
    let content = r#"// @noLib: true
/*1*/this;
export const c = 1;"#;
    let mut s = Session::new_for_test("findAllRefsGlobalThisKeywordInModule", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1")
}
