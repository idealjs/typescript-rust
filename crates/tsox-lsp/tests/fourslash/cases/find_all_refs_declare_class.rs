use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_declare_class() {
    let content = r#"/*1*/declare class /*2*/C {
    static m(): void;
}"#;
    let _s = Session::new_for_test("findAllRefsDeclareClass", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}
