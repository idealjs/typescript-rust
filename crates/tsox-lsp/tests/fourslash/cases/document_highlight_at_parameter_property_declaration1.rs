use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn document_highlight_at_parameter_property_declaration1() {
    let content = r#"// @Filename: file1.ts
class Foo {
    constructor(private [|privateParam|]: number,
        public [|publicParam|]: string,
        protected [|protectedParam|]: boolean) {

        let localPrivate = [|privateParam|];
        this.[|privateParam|] += 10;

        let localPublic = [|publicParam|];
        this.[|publicParam|] += " Hello!";

        let localProtected = [|protectedParam|];
        this.[|protectedParam|] = false;
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
