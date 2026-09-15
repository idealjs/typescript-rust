use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_class_expression_static() {
    let content = r#"let A = class Foo {
    public [|static|] foo;
    [|static|] a;
    constructor(public y: string, private x: string) {
    }
    public method() { }
    private method2() {}
    public [|static|] static() { }
    private [|static|] static2() { }
}

let B = class D {
    static a;
    constructor(private x: number) {
    }
    private static test() {}
    public static test2() {}
}"#;
    let _s = Session::new_for_test("getOccurrencesClassExpressionStatic", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
