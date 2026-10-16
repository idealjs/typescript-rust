use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("findAllRefsClassStaticBlocks", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "classStaticBocks1", "classStaticBocks2", "classStaticBocks3")
}
