use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_parameter_property_declaration1() {
    let content = r#"class Foo {
    constructor(private /*1*/privateParam: number) {
        let localPrivate = privateParam;
        this.privateParam += 10;
    }
}"#;
    let _s = Session::new_for_test("findAllRefsParameterPropertyDeclaration1", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
