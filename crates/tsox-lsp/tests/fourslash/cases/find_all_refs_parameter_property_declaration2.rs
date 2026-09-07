use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_parameter_property_declaration2() {
    let content = r#"class Foo {
    constructor(public /*0*/publicParam: number) {
        let localPublic = /*1*/publicParam;
        this./*2*/publicParam += 10;
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "0", "1", "2")
}
