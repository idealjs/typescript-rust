use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
