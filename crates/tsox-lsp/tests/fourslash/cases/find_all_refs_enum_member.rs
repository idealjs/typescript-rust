use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_enum_member() {
    let content = r#"enum E { /*1*/A, B }
const e: E./*2*/A = E./*3*/A;"#;
    let _s = Session::new_for_test("findAllRefsEnumMember", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
