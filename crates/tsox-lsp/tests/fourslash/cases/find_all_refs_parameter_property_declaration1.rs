use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_parameter_property_declaration1() {
    let content = r#"class Foo {
    constructor(private /*1*/privateParam: number) {
        let localPrivate = privateParam;
        this.privateParam += 10;
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1")
}
