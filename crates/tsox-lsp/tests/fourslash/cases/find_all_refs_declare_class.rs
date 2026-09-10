use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_declare_class() {
    let content = r#"/*1*/declare class /*2*/C {
    static m(): void;
}"#;
    let mut s = Session::new_for_test("findAllRefsDeclareClass", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2")
}
