use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_parameter_property_declaration2() {
    let content = r#"class Foo {
    constructor(public /*0*/publicParam: number) {
        let localPublic = /*1*/publicParam;
        this./*2*/publicParam += 10;
    }
}"#;
    let _s = Session::new_for_test("findAllRefsParameterPropertyDeclaration2", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "0", "1", "2")
}
