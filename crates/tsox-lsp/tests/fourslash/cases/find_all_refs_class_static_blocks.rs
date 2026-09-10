use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_class_static_blocks() {
    let content = r#"class ClassStaticBocks {
    static x;
    [|[|/*classStaticBocks1*/static|] {}|]
    static y;
    [|[|/*classStaticBocks2*/static|] {}|]
    static y;
    [|[|/*classStaticBocks3*/static|] {}|]
}"#;
    let mut s = Session::new_for_test("findAllRefsClassStaticBlocks", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "classStaticBocks1", "classStaticBocks2", "classStaticBocks3")
}
