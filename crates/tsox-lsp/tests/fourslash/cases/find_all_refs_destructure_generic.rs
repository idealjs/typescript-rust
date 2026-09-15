use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_destructure_generic() {
    let content = r#"interface I<T> {
    /*0*/x: boolean;
}
declare const i: I<number>;
const { /*1*/x } = i;"#;
    let _s = Session::new_for_test("findAllRefsDestructureGeneric", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "0", "1")
}
