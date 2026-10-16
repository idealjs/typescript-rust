use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_enum_as_namespace() {
    let content = r#"/*1*/enum /*2*/E { A }
let e: /*3*/E.A;"#;
    let _s = Session::new_for_test("findAllRefsEnumAsNamespace", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
