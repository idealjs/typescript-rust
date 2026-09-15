use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_for_rest() {
    let content = r#"interface Gen {
    x: number
    /*1*/parent: Gen;
    millenial: string;
}
let t: Gen;
var { x, ...rest } = t;
rest./*2*/parent;"#;
    let _s = Session::new_for_test("findAllRefsForRest", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}
