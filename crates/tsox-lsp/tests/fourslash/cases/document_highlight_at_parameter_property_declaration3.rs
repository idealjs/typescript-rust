use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn document_highlight_at_parameter_property_declaration3() {
    let content = r#"// @Filename: file1.ts
class Foo {
    // This is not valid syntax: parameter property can't be binding pattern
    constructor(private [[|privateParam|]]: number,
        public [[|publicParam|]]: string,
        protected [[|protectedParam|]]: boolean) {

        let localPrivate = [|privateParam|];
        this.privateParam += 10;

        let localPublic = [|publicParam|];
        this.publicParam += " Hello!";

        let localProtected = [|protectedParam|];
        this.protectedParam = false;
    }
}"#;
    let mut s = Session::new_for_test("documentHighlightAtParameterPropertyDeclaration3", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
