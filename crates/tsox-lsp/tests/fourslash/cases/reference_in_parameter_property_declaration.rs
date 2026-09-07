use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn reference_in_parameter_property_declaration() {
    let content = r#"// @Filename: file1.ts
class Foo {
    constructor(private /*1*/privateParam: number,
        public /*2*/publicParam: string,
        protected /*3*/protectedParam: boolean) {

        let localPrivate = privateParam;
        this.privateParam += 10;

        let localPublic = publicParam;
        this.publicParam += " Hello!";

        let localProtected = protectedParam;
        this.protectedParam = false;
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
