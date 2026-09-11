use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_occurrences_class_expression_private() {
    let content = r#"let A = class Foo {
    [|private|] foo;
    [|private|] private;
    constructor([|private|] y: string, public x: string) {
    }
    [|private|] method() { }
    public method2() { }
    [|private|] static static() { }
}

let B = class D {
    constructor(private x: number) {
    }
    private test() {}
    public test2() {}
}"#;
    let mut s = Session::new_for_test("getOccurrencesClassExpressionPrivate", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
