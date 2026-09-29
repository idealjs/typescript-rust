use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("documentHighlightAtParameterPropertyDeclaration1", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
