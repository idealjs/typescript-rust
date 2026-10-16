use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_class_expression_constructor() {
    let content = r#"let A = class Foo {
    [|constructor|]();
    [|constructor|](x: number);
    [|constructor|](y: string);
    [|constructor|](a?: any) {
    }
}

let B = class D {
    constructor(x: number) {
    }
}"#;
    let _s = Session::new_for_test("getOccurrencesClassExpressionConstructor", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
