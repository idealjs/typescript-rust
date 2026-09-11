use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_occurrences_class_expression_public() {
    let content = r#"let A = class Foo {
    [|public|] foo;
    [|public|] public;
    constructor([|public|] y: string, private x: string) {
    }
    [|public|] method() { }
    private method2() {}
    [|public|] static static() { }
}

let B = class D {
    constructor(private x: number) {
    }
    private test() {}
    public test2() {}
}"#;
    let mut s = Session::new_for_test("getOccurrencesClassExpressionPublic", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
