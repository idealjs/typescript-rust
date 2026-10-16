use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_destructure_getter() {
    let content = r#"class Test {
    get /*x0*/x() { return 0; }

    set /*y0*/y(a: number) {}
}
const { /*x1*/x, /*y1*/y } = new Test();
/*x2*/x; /*y2*/y;"#;
    let _s = Session::new_for_test("findAllRefsDestructureGetter", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "x0", "x1", "x2", "y0", "y1", "y2")
}
