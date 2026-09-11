use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_parameter_property_declaration3() {
    let content = r#"class Foo {
    constructor(protected /*0*/protectedParam: number) {
        let localProtected = /*1*/protectedParam;
        this./*2*/protectedParam += 10;
    }
}"#;
    let mut s = Session::new_for_test("findAllRefsParameterPropertyDeclaration3", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "0", "1", "2")
}
