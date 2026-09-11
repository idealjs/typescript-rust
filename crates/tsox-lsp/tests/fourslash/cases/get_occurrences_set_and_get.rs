use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_occurrences_set_and_get() {
    let content = r#"class Foo {
    [|set|] bar(b: any) {
    }

    public [|get|] bar(): any {
        return undefined;
    }

    public set set(s: any) {
    }

    public get set(): any {
        return undefined;
    }

    public set get(g: any) {
    }

    public get get(): any {
        return undefined;
    }
}"#;
    let mut s = Session::new_for_test("getOccurrencesSetAndGet", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
